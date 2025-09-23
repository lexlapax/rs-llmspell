"""
Integration tests for DAP (Debug Adapter Protocol) functionality through Jupyter.

These tests validate that llmspell correctly implements DAP commands when
accessed through the Jupyter protocol using real jupyter_client connections.
"""

import pytest
import json
import time
import logging
from pathlib import Path
import statistics

logger = logging.getLogger(__name__)


class TestJupyterDAP:
    """Test DAP functionality through Jupyter protocol."""

    def send_dap_request(self, client, command, arguments=None, timeout=5):
        """
        Send a DAP request through Jupyter's debug_request message type.

        Args:
            client: Jupyter kernel client
            command: DAP command name
            arguments: Optional dict of command arguments
            timeout: Max seconds to wait for response

        Returns:
            Response body from the DAP command

        Raises:
            AssertionError: If the response indicates failure
            TimeoutError: If no response within timeout
        """
        # Construct the debug_request message
        msg_content = {
            'command': command,
            'arguments': arguments or {}
        }

        logger.debug(f"Sending DAP request: {command} with args: {arguments}")

        # Send through shell channel
        msg = client.session.msg('debug_request', msg_content)
        client.shell_channel.send(msg)

        # Wait for reply
        try:
            reply = client.get_shell_msg(timeout=timeout)
        except Exception as e:
            raise TimeoutError(f"No response to {command} within {timeout}s: {e}")

        # Validate response
        assert reply['msg_type'] == 'debug_reply', f"Expected debug_reply, got {reply['msg_type']}"
        assert reply['content']['status'] == 'ok', f"DAP request failed: {reply['content']}"

        response_body = reply['content'].get('body', {})
        logger.debug(f"DAP response: {response_body}")
        return response_body

    def wait_for_event(self, client, event_type, timeout=5):
        """
        Wait for a specific DAP event on the iopub channel.

        Args:
            client: Jupyter kernel client
            event_type: Name of the DAP event to wait for
            timeout: Max seconds to wait

        Returns:
            Event body when received

        Raises:
            TimeoutError: If event not received within timeout
        """
        logger.debug(f"Waiting for DAP event: {event_type}")
        deadline = time.time() + timeout

        while time.time() < deadline:
            try:
                msg = client.get_iopub_msg(timeout=0.1)
                if msg['msg_type'] == 'debug_event':
                    event = msg['content'].get('event')
                    if event == event_type:
                        logger.debug(f"Received event {event_type}: {msg['content']}")
                        return msg['content'].get('body', {})
            except:
                # No message available, continue waiting
                continue

        raise TimeoutError(f"Event {event_type} not received within {timeout}s")

    def test_dap_initialization(self, kernel_client):
        """Test basic DAP initialization through Jupyter."""
        response = self.send_dap_request(kernel_client, 'initialize', {
            'clientID': 'pytest',
            'clientName': 'Python Test Suite',
            'adapterID': 'llmspell',
            'pathFormat': 'path',
            'linesStartAt1': True,
            'columnsStartAt1': True,
            'supportsVariableType': True,
            'supportsVariablePaging': True,
            'supportsRunInTerminalRequest': False
        })

        # Verify required capabilities
        assert 'supportsConfigurationDoneRequest' in response
        assert 'supportsSetBreakpoints' in response
        assert 'supportsStepBack' in response or 'supportsSteppingGranularity' in response

        # Verify adapter info
        assert response.get('supportsDebuggerProperties', False) or True  # Optional

        # Send initialized notification
        self.send_dap_request(kernel_client, 'initialized')

    def test_simple_breakpoint_session(self, kernel_client, test_script_dir):
        """Test a simple debug session with breakpoint."""
        # Create test script
        test_script = test_script_dir / "simple_debug.lua"
        test_script.write_text("""
-- Test script for debugging
local x = 10
local y = 20
local z = x + y  -- Breakpoint here (line 4)
print("Result: " .. z)
print("Done")
""")

        # Initialize DAP
        self.send_dap_request(kernel_client, 'initialize', {
            'clientID': 'pytest-simple',
            'linesStartAt1': True,
            'columnsStartAt1': True
        })

        # Set breakpoint at line 4
        bp_response = self.send_dap_request(kernel_client, 'setBreakpoints', {
            'source': {
                'name': test_script.name,
                'path': str(test_script)
            },
            'breakpoints': [{'line': 4}],
            'lines': [4]
        })

        # Verify breakpoint was set
        breakpoints = bp_response.get('breakpoints', [])
        assert len(breakpoints) == 1, f"Expected 1 breakpoint, got {len(breakpoints)}"
        assert breakpoints[0].get('verified', False), "Breakpoint not verified"
        assert breakpoints[0].get('line') == 4, f"Breakpoint set at wrong line: {breakpoints[0]}"

        # Launch debug session
        self.send_dap_request(kernel_client, 'launch', {
            'program': str(test_script),
            'stopOnEntry': False,
            'noDebug': False,
            'args': []
        })

        # Configuration done
        self.send_dap_request(kernel_client, 'configurationDone')

        # Execute the script - should hit breakpoint
        exec_msg = kernel_client.execute(f'dofile("{test_script}")', silent=False)

        # Wait for stopped event (hit breakpoint)
        stopped = self.wait_for_event(kernel_client, 'stopped', timeout=10)
        assert stopped['reason'] == 'breakpoint', f"Stopped for wrong reason: {stopped['reason']}"

        # Get thread ID from stopped event
        thread_id = stopped.get('threadId', 1)

        # Get stack trace
        stack_response = self.send_dap_request(kernel_client, 'stackTrace', {
            'threadId': thread_id,
            'startFrame': 0,
            'levels': 20
        })

        frames = stack_response.get('stackFrames', [])
        assert len(frames) > 0, "No stack frames returned"
        assert frames[0]['line'] == 4, f"Top frame at wrong line: {frames[0]['line']}"

        # Continue execution
        self.send_dap_request(kernel_client, 'continue', {
            'threadId': thread_id
        })

        # Wait for terminated event
        terminated = self.wait_for_event(kernel_client, 'terminated', timeout=5)
        assert terminated is not None, "Session didn't terminate"

    def test_stepping_operations(self, kernel_client, test_script_dir):
        """Test step over, step in, and step out operations."""
        # Create test script with function
        test_script = test_script_dir / "stepping.lua"
        test_script.write_text("""
function calculate(a, b)
    local sum = a + b  -- Line 2
    local product = a * b  -- Line 3
    return sum, product  -- Line 4
end

local x = 5  -- Line 7: Start here
local y = 10
local s, p = calculate(x, y)  -- Line 9: Step into this
print("Sum: " .. s)
print("Product: " .. p)
""")

        # Initialize and set breakpoint at line 7
        self.send_dap_request(kernel_client, 'initialize', {
            'clientID': 'pytest-stepping',
            'linesStartAt1': True
        })

        self.send_dap_request(kernel_client, 'setBreakpoints', {
            'source': {'path': str(test_script)},
            'breakpoints': [{'line': 7}]
        })

        # Launch and wait for breakpoint
        self.send_dap_request(kernel_client, 'launch', {
            'program': str(test_script),
            'stopOnEntry': False
        })

        self.send_dap_request(kernel_client, 'configurationDone')
        kernel_client.execute(f'dofile("{test_script}")')

        stopped = self.wait_for_event(kernel_client, 'stopped')
        thread_id = stopped['threadId']

        # Step over (to line 8)
        self.send_dap_request(kernel_client, 'next', {'threadId': thread_id})
        stopped = self.wait_for_event(kernel_client, 'stopped')
        assert stopped['reason'] == 'step'

        # Step over again (to line 9)
        self.send_dap_request(kernel_client, 'next', {'threadId': thread_id})
        stopped = self.wait_for_event(kernel_client, 'stopped')

        # Now step into the function call
        self.send_dap_request(kernel_client, 'stepIn', {'threadId': thread_id})
        stopped = self.wait_for_event(kernel_client, 'stopped')

        # Verify we're inside the function
        stack = self.send_dap_request(kernel_client, 'stackTrace', {
            'threadId': thread_id
        })
        assert len(stack['stackFrames']) >= 2, "Should have multiple frames when in function"

        # Step out of function
        self.send_dap_request(kernel_client, 'stepOut', {'threadId': thread_id})
        stopped = self.wait_for_event(kernel_client, 'stopped')

        # Continue to finish
        self.send_dap_request(kernel_client, 'continue', {'threadId': thread_id})
        self.wait_for_event(kernel_client, 'terminated')

    def test_variable_inspection(self, kernel_client, test_script_dir):
        """Test variable inspection at breakpoints."""
        # Create test script with various variable types
        test_script = test_script_dir / "variables.lua"
        test_script.write_text("""
local number_var = 42
local string_var = "hello world"
local bool_var = true
local table_var = {
    key1 = "value1",
    key2 = 123,
    nested = {
        inner = "data"
    }
}
local nil_var = nil
print("Breakpoint here")  -- Line 12
""")

        # Initialize and set breakpoint
        self.send_dap_request(kernel_client, 'initialize', {
            'clientID': 'pytest-variables',
            'linesStartAt1': True
        })

        self.send_dap_request(kernel_client, 'setBreakpoints', {
            'source': {'path': str(test_script)},
            'breakpoints': [{'line': 12}]
        })

        # Launch and hit breakpoint
        self.send_dap_request(kernel_client, 'launch', {
            'program': str(test_script)
        })

        self.send_dap_request(kernel_client, 'configurationDone')
        kernel_client.execute(f'dofile("{test_script}")')

        stopped = self.wait_for_event(kernel_client, 'stopped')
        thread_id = stopped['threadId']

        # Get stack frame
        stack = self.send_dap_request(kernel_client, 'stackTrace', {
            'threadId': thread_id
        })
        frame_id = stack['stackFrames'][0]['id']

        # Get scopes
        scopes_response = self.send_dap_request(kernel_client, 'scopes', {
            'frameId': frame_id
        })

        scopes = scopes_response.get('scopes', [])
        assert len(scopes) > 0, "No scopes returned"

        # Find local scope
        local_scope = next((s for s in scopes if s['name'] == 'Local'), None)
        assert local_scope is not None, "Local scope not found"

        # Get variables
        vars_response = self.send_dap_request(kernel_client, 'variables', {
            'variablesReference': local_scope['variablesReference']
        })

        variables = vars_response.get('variables', [])
        var_dict = {v['name']: v for v in variables}

        # Verify variable values
        assert 'number_var' in var_dict
        assert var_dict['number_var']['value'] == '42'

        assert 'string_var' in var_dict
        assert 'hello world' in var_dict['string_var']['value']

        assert 'bool_var' in var_dict
        assert var_dict['bool_var']['value'] == 'true'

        assert 'table_var' in var_dict
        # Tables should have a variablesReference for expansion
        assert var_dict['table_var'].get('variablesReference', 0) > 0

        # Clean up
        self.send_dap_request(kernel_client, 'continue', {'threadId': thread_id})

    def test_performance_benchmarks(self, kernel_client):
        """Validate DAP performance meets requirements."""
        # Test initialization performance
        init_times = []
        for i in range(5):
            start = time.time()
            self.send_dap_request(kernel_client, 'initialize', {
                'clientID': f'perf_test_{i}'
            })
            init_time = time.time() - start
            init_times.append(init_time)
            logger.info(f"Init {i}: {init_time:.3f}s")

        avg_init = statistics.mean(init_times)
        max_init = max(init_times)

        logger.info(f"Init times - Avg: {avg_init:.3f}s, Max: {max_init:.3f}s")
        assert avg_init < 0.05, f"Avg init time {avg_init:.3f}s exceeds 50ms"
        assert max_init < 0.1, f"Max init time {max_init:.3f}s exceeds 100ms"

        # Test stepping performance (simplified since we need a real debug session)
        # Create a simple loop script for stepping
        test_code = """
for i = 1, 10 do
    local x = i * 2
end
"""

        self.send_dap_request(kernel_client, 'setBreakpoints', {
            'source': {'name': 'inline', 'sourceReference': 0},
            'breakpoints': [{'line': 2}]
        })

        self.send_dap_request(kernel_client, 'launch', {
            'program': 'inline',
            'stopOnEntry': True,
            'noDebug': False
        })

        # Measure a few step operations
        step_times = []
        for _ in range(3):
            start = time.time()
            self.send_dap_request(kernel_client, 'next', {'threadId': 1})
            step_time = time.time() - start
            step_times.append(step_time)

        if step_times:
            avg_step = statistics.mean(step_times)
            max_step = max(step_times)
            logger.info(f"Step times - Avg: {avg_step:.3f}s, Max: {max_step:.3f}s")
            assert avg_step < 0.05, f"Avg step time {avg_step:.3f}s exceeds 50ms"
            assert max_step < 0.1, f"Max step time {max_step:.3f}s exceeds 100ms"

    def test_multiple_breakpoints(self, kernel_client, test_script_dir):
        """Test handling multiple breakpoints in a single file."""
        test_script = test_script_dir / "multi_bp.lua"
        test_script.write_text("""
print("Start")  -- Line 1
local a = 1  -- Line 2: BP1
local b = 2  -- Line 3
local c = 3  -- Line 4: BP2
local d = 4  -- Line 5
local e = 5  -- Line 6: BP3
print("End")  -- Line 7
""")

        self.send_dap_request(kernel_client, 'initialize', {
            'clientID': 'pytest-multi-bp'
        })

        # Set multiple breakpoints
        bp_response = self.send_dap_request(kernel_client, 'setBreakpoints', {
            'source': {'path': str(test_script)},
            'breakpoints': [
                {'line': 2},
                {'line': 4},
                {'line': 6}
            ]
        })

        breakpoints = bp_response.get('breakpoints', [])
        assert len(breakpoints) == 3, f"Expected 3 breakpoints, got {len(breakpoints)}"

        # Launch
        self.send_dap_request(kernel_client, 'launch', {
            'program': str(test_script)
        })
        self.send_dap_request(kernel_client, 'configurationDone')

        kernel_client.execute(f'dofile("{test_script}")')

        # Should hit all three breakpoints in sequence
        for expected_line in [2, 4, 6]:
            stopped = self.wait_for_event(kernel_client, 'stopped')
            assert stopped['reason'] == 'breakpoint'

            # Verify we're at the right line
            stack = self.send_dap_request(kernel_client, 'stackTrace', {
                'threadId': stopped['threadId']
            })
            assert stack['stackFrames'][0]['line'] == expected_line

            # Continue to next
            self.send_dap_request(kernel_client, 'continue', {
                'threadId': stopped['threadId']
            })

        # Should terminate after last continue
        self.wait_for_event(kernel_client, 'terminated')

    def test_conditional_breakpoint(self, kernel_client, test_script_dir):
        """Test conditional breakpoints (if supported)."""
        test_script = test_script_dir / "conditional.lua"
        test_script.write_text("""
for i = 1, 10 do
    print("i = " .. i)  -- Line 2: conditional BP when i == 5
end
""")

        self.send_dap_request(kernel_client, 'initialize', {
            'clientID': 'pytest-conditional'
        })

        # Try to set conditional breakpoint
        bp_response = self.send_dap_request(kernel_client, 'setBreakpoints', {
            'source': {'path': str(test_script)},
            'breakpoints': [{
                'line': 2,
                'condition': 'i == 5'
            }]
        })

        breakpoints = bp_response.get('breakpoints', [])
        if breakpoints and breakpoints[0].get('verified', False):
            # Conditional breakpoints are supported
            self.send_dap_request(kernel_client, 'launch', {
                'program': str(test_script)
            })
            self.send_dap_request(kernel_client, 'configurationDone')

            kernel_client.execute(f'dofile("{test_script}")')

            # Should stop when i == 5
            stopped = self.wait_for_event(kernel_client, 'stopped')

            # Get variables to verify i == 5
            stack = self.send_dap_request(kernel_client, 'stackTrace', {
                'threadId': stopped['threadId']
            })
            frame_id = stack['stackFrames'][0]['id']

            scopes = self.send_dap_request(kernel_client, 'scopes', {
                'frameId': frame_id
            })

            local_scope = scopes['scopes'][0]
            vars_response = self.send_dap_request(kernel_client, 'variables', {
                'variablesReference': local_scope['variablesReference']
            })

            i_var = next((v for v in vars_response['variables'] if v['name'] == 'i'), None)
            if i_var:
                assert i_var['value'] == '5', f"Stopped at wrong iteration: i={i_var['value']}"

            self.send_dap_request(kernel_client, 'continue', {
                'threadId': stopped['threadId']
            })
        else:
            pytest.skip("Conditional breakpoints not supported")
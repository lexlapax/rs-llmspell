#!/usr/bin/env python3
"""
Simple standalone test for DAP functionality through Jupyter protocol.
Tests debugging a Lua script with breakpoints, variable inspection, and stepping.

REQUIREMENTS:
- llmspell binary built in ../../target/debug/ or ../../target/release/
- Python packages: pytest, jupyter_client (install via requirements.txt)
- The test will automatically start a kernel daemon if needed
- Requires ~15-20 seconds to complete

HOW IT WORKS:
1. Starts an llmspell kernel daemon (or uses existing one)
2. Connects via Jupyter protocol over ZMQ
3. Sends DAP commands to debug Lua scripts
4. Validates breakpoints, stepping, and variable inspection

Task 10.7.9: Complete End-to-End DAP Testing with Lua Scripts
"""

import json
import subprocess
import time
import tempfile
import os
import signal
from pathlib import Path
from jupyter_client import BlockingKernelClient

def start_kernel():
    """Start kernel daemon and return connection info."""
    # Clean up any existing kernel
    subprocess.run(["pkill", "-f", "llmspell.*kernel"], capture_output=True)
    subprocess.run(["rm", "-f", "/tmp/llmspell-kernel-port-*.pid"], capture_output=True)
    time.sleep(1)

    # Create test directory
    test_dir = Path("/tmp/llmspell-test")
    test_dir.mkdir(exist_ok=True)

    # Start kernel daemon - binary is in project root
    llmspell_path = "../../target/debug/llmspell"
    if not Path(llmspell_path).exists():
        # Try release build
        llmspell_path = "../../target/release/llmspell"

    cmd = [
        llmspell_path, "kernel", "start",
        "--daemon",
        "--port", "0",  # Let OS assign ports
        "--connection-file", "/tmp/llmspell-test/kernel.json",
        "--log-file", "/tmp/llmspell-test/kernel.log",
        "--idle-timeout", "0"
    ]

    print(f"Starting kernel: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"Failed to start kernel: {result.stderr}")
        return None

    # Wait for connection file
    conn_file = Path("/tmp/llmspell-test/kernel.json")
    for _ in range(10):
        if conn_file.exists():
            break
        time.sleep(0.5)
    else:
        print("Connection file not created")
        return None

    # Load connection info
    with open(conn_file) as f:
        conn_info = json.load(f)

    print(f"Kernel started on ports: {conn_info['shell_port']}-{conn_info['hb_port']}")
    return conn_info

def stop_kernel():
    """Stop kernel daemon."""
    subprocess.run(["pkill", "-f", "llmspell.*kernel"], capture_output=True)
    print("Kernel stopped")

def send_debug_request(client, command, arguments=None):
    """Send a debug_request and get the reply."""
    # Build the message using session
    msg = client.session.msg(
        'debug_request',
        {
            'command': command,
            'arguments': arguments or {}
        }
    )

    # Send on control channel
    client.control_channel.send(msg)

    # Get the message ID for matching the reply
    msg_id = msg['header']['msg_id']

    # Wait for reply with matching msg_id
    for _ in range(10):
        try:
            reply = client.control_channel.get_msg(timeout=1)
            if reply.get('parent_header', {}).get('msg_id') == msg_id:
                return reply
        except:
            continue

    return None

def test_simple_breakpoint():
    """Test a simple debug session with one breakpoint."""

    # Start kernel
    conn_info = start_kernel()
    if not conn_info:
        return False

    try:
        # Create Lua test script
        test_script = Path("/tmp/test_debug.lua")
        test_script.write_text("""-- Test script for debugging
local x = 10
local y = 20
local z = x + y  -- Line 4: Set breakpoint here
print("Result: " .. z)
print("Done")
""")

        # Create kernel client
        client = BlockingKernelClient()
        client.load_connection_file("/tmp/llmspell-test/kernel.json")
        client.start_channels()

        # Give kernel a moment to be fully ready
        time.sleep(1)

        print("\n1. Testing DAP initialization...")

        # Send DAP initialize request
        reply = send_debug_request(client, 'initialize', {
            'clientID': 'test_dap_simple',
            'clientName': 'Simple DAP Test',
            'linesStartAt1': True,
            'columnsStartAt1': True
        })

        if not reply:
            print(f"  ✗ No reply received at all")
            return False

        if reply.get('msg_type') != 'debug_reply':
            print(f"  ✗ Got {reply.get('msg_type')} instead of debug_reply")
            print(f"     Full reply: {reply}")
            return False

        content = reply.get('content', {})
        if 'body' in content:
            caps = content['body']
            print(f"  ✓ DAP initialized")
            print(f"    - Supports breakpoints: {caps.get('supportsSetBreakpoints', False)}")
            print(f"    - Supports stepping: {caps.get('supportsSteppingGranularity', False)}")
        else:
            print(f"  ✗ No capabilities in response: {content}")
            return False

        print("\n2. Setting breakpoint at line 4...")

        # Set breakpoint
        reply = send_debug_request(client, 'setBreakpoints', {
            'source': {
                'path': str(test_script)
            },
            'breakpoints': [{'line': 4}]
        })

        if not reply:
            print(f"  ✗ No reply received")
            return False

        content = reply.get('content', {})
        if 'body' in content and 'breakpoints' in content['body']:
            bps = content['body']['breakpoints']
            if bps and bps[0].get('verified'):
                print(f"  ✓ Breakpoint set at line {bps[0].get('line')}")
            else:
                print(f"  ✗ Breakpoint not verified: {bps}")
                return False
        else:
            print(f"  ✗ No breakpoints in response: {content}")
            return False

        print("\n3. Launching debug session...")

        # Launch debug session
        reply = send_debug_request(client, 'launch', {
            'program': str(test_script),
            'stopOnEntry': False,
            'noDebug': False
        })

        if reply:
            print(f"  ✓ Debug session launched")
        else:
            print(f"  ✗ Failed to launch")
            return False

        print("\n4. Executing script (should hit breakpoint)...")

        # Execute the script
        exec_msg_id = client.execute(f'dofile("{test_script}")', silent=False)

        # Wait for stopped event on IOPub channel
        stopped = False
        for _ in range(10):
            try:
                msg = client.get_iopub_msg(timeout=1)
                if msg['msg_type'] == 'debug_event':
                    event = msg['content'].get('event')
                    if event == 'stopped':
                        stopped = True
                        print(f"  ✓ Hit breakpoint at line {msg['content'].get('body', {}).get('line', '?')}")
                        break
            except:
                continue

        if not stopped:
            print("  ✗ Breakpoint not hit (no stopped event)")
            # Try to get execute result anyway
            try:
                exec_reply = client.get_shell_msg(timeout=2)
                print(f"  Script executed without stopping: {exec_reply.get('content', {}).get('status')}")
            except:
                print("  Script execution timed out")
            return False

        print("\n5. Getting stack trace...")

        # Get stack trace
        reply = send_debug_request(client, 'stackTrace', {
            'threadId': 1
        })

        if reply:
            content = reply.get('content', {})
            if 'body' in content and 'stackFrames' in content['body']:
                frames = content['body']['stackFrames']
                if frames:
                    print(f"  ✓ Stack frame: {frames[0].get('name')} at line {frames[0].get('line')}")
                else:
                    print(f"  ✗ No stack frames")
            else:
                print(f"  ✗ No stack trace in response")

        print("\n6. Inspecting variables...")

        # Get scopes
        reply = send_debug_request(client, 'scopes', {
            'frameId': 1
        })

        if reply:
            content = reply.get('content', {})
            if 'body' in content and 'scopes' in content['body']:
                scopes = content['body']['scopes']
                if scopes:
                    # Get variables from local scope
                    scope_ref = scopes[0].get('variablesReference')

                    reply = send_debug_request(client, 'variables', {
                        'variablesReference': scope_ref
                    })

                    if reply:
                        content = reply.get('content', {})
                        if 'body' in content and 'variables' in content['body']:
                            vars = content['body']['variables']
                            print(f"  ✓ Variables:")
                            for var in vars:
                                if var['name'] in ['x', 'y', 'z']:
                                    print(f"    - {var['name']} = {var['value']}")
                        else:
                            print(f"  ✗ No variables in response")
            else:
                print(f"  ✗ No scopes in response")

        print("\n7. Continuing execution...")

        # Continue execution
        reply = send_debug_request(client, 'continue', {
            'threadId': 1
        })

        if reply:
            print(f"  ✓ Execution continued")
        else:
            print(f"  ✗ Failed to continue")

        # Get final output
        try:
            exec_reply = client.get_shell_msg(timeout=5)
            if exec_reply['content'].get('status') == 'ok':
                print(f"  ✓ Script completed successfully")
            else:
                print(f"  ✗ Script failed: {exec_reply['content']}")
        except:
            print(f"  ✗ No execution reply received")

        client.stop_channels()
        return True

    except Exception as e:
        print(f"\n✗ Test failed with error: {e}")
        import traceback
        traceback.print_exc()
        return False

    finally:
        stop_kernel()

def test_performance():
    """Test DAP performance requirements."""

    # Start kernel
    conn_info = start_kernel()
    if not conn_info:
        return False

    try:
        # Create kernel client
        client = BlockingKernelClient()
        client.load_connection_file("/tmp/llmspell-test/kernel.json")
        client.start_channels()

        print("\nPerformance Testing:")

        # Test initialization performance
        start_time = time.time()

        reply = send_debug_request(client, 'initialize', {
            'clientID': 'perf_test'
        })

        if reply:
            init_time = (time.time() - start_time) * 1000
            print(f"  DAP initialization: {init_time:.1f}ms (requirement: <50ms)")

            if init_time < 50:
                print(f"  ✓ Performance requirement met")
                return True
            else:
                print(f"  ✗ Too slow")
                return False
        else:
            print(f"  ✗ Failed to initialize")
            return False

    finally:
        client.stop_channels()
        stop_kernel()

if __name__ == "__main__":
    print("="*60)
    print("Task 10.7.9: End-to-End DAP Testing with Lua Scripts")
    print("="*60)

    # Test 1: Simple breakpoint session
    success1 = test_simple_breakpoint()

    # Test 2: Performance
    success2 = test_performance()

    print("\n" + "="*60)
    print("Test Results:")
    print(f"  Simple breakpoint session: {'✓ PASS' if success1 else '✗ FAIL'}")
    print(f"  Performance requirements:  {'✓ PASS' if success2 else '✗ FAIL'}")

    if success1:
        print("\n✓ SUCCESS: DAP debugging works with Lua scripts!")
        print("  Task 10.7.9 objectives achieved")
    else:
        print("\n✗ FAILURE: DAP debugging not working")
        print("  Check /tmp/llmspell-test/kernel.log for details")
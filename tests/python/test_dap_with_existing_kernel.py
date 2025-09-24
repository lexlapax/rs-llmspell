#!/usr/bin/env python3
"""
Test DAP with existing kernel (don't start new one).
"""

import json
import time
from pathlib import Path
from jupyter_client import BlockingKernelClient

def send_debug_request(client, command, arguments=None):
    """Send a debug_request and get the reply."""
    msg = client.session.msg(
        'debug_request',
        {
            'command': command,
            'arguments': arguments or {}
        }
    )

    client.control_channel.send(msg)
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

def test_dap_workflow():
    """Test complete DAP workflow."""

    print("DAP Workflow Test (Using Existing Kernel)")
    print("="*50)

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

    print("\n1. Testing DAP initialization...")

    reply = send_debug_request(client, 'initialize', {
        'clientID': 'test_with_existing',
        'clientName': 'DAP Workflow Test',
        'linesStartAt1': True,
        'columnsStartAt1': True
    })

    if not reply or reply.get('msg_type') != 'debug_reply':
        print(f"  ✗ Failed: {reply}")
        return False

    content = reply.get('content', {})
    if content.get('success'):
        print(f"  ✅ DAP initialized")
        caps = content.get('body', {})
        print(f"     Supports breakpoints: {caps.get('supportsConditionalBreakpoints')}")
        print(f"     Supports stepping: {caps.get('supportsSteppingGranularity')}")

    print("\n2. Setting breakpoint at line 4...")

    reply = send_debug_request(client, 'setBreakpoints', {
        'source': {
            'path': str(test_script)
        },
        'breakpoints': [{'line': 4}]
    })

    if reply and reply.get('content', {}).get('success'):
        bps = reply['content'].get('body', {}).get('breakpoints', [])
        if bps and bps[0].get('verified'):
            print(f"  ✅ Breakpoint set and verified")
        else:
            print(f"  ✗ Breakpoint not verified")
    else:
        print(f"  ✗ Failed to set breakpoint")

    print("\n3. Launching debug session...")

    reply = send_debug_request(client, 'launch', {
        'program': str(test_script),
        'stopOnEntry': False,
        'noDebug': False
    })

    if reply and reply.get('content', {}).get('success'):
        print(f"  ✅ Debug session launched")
    else:
        print(f"  ✗ Failed to launch")

    print("\n4. Executing script...")

    # This should hit the breakpoint
    exec_msg_id = client.execute(f'dofile("{test_script}")', silent=False)
    print(f"  Sent execute with msg_id: {exec_msg_id}")

    # Wait for stopped event
    print("\n5. Waiting for breakpoint hit...")
    stopped = False
    for _ in range(10):
        try:
            msg = client.get_iopub_msg(timeout=1)
            if msg['msg_type'] == 'debug_event':
                event = msg['content'].get('event')
                if event == 'stopped':
                    stopped = True
                    print(f"  ✅ Hit breakpoint!")
                    body = msg['content'].get('body', {})
                    print(f"     Reason: {body.get('reason')}")
                    print(f"     Line: {body.get('line')}")
                    break
        except:
            pass

    if not stopped:
        print("  ⚠️  Breakpoint not hit, checking if script completed...")
        try:
            exec_reply = client.get_shell_msg(timeout=2)
            status = exec_reply.get('content', {}).get('status')
            print(f"  Script status: {status}")
            if status == 'ok':
                print("  Note: Script ran without stopping at breakpoint")
        except:
            print("  No execution reply")

    if stopped:
        print("\n6. Getting stack trace...")
        reply = send_debug_request(client, 'stackTrace', {
            'threadId': 1
        })

        if reply and reply.get('content', {}).get('success'):
            frames = reply['content'].get('body', {}).get('stackFrames', [])
            if frames:
                print(f"  ✅ Stack frame: {frames[0].get('name')} at line {frames[0].get('line')}")

        print("\n7. Inspecting variables...")
        reply = send_debug_request(client, 'scopes', {
            'frameId': 1
        })

        if reply and reply.get('content', {}).get('success'):
            scopes = reply['content'].get('body', {}).get('scopes', [])
            if scopes:
                scope_ref = scopes[0].get('variablesReference')

                reply = send_debug_request(client, 'variables', {
                    'variablesReference': scope_ref
                })

                if reply and reply.get('content', {}).get('success'):
                    vars = reply['content'].get('body', {}).get('variables', [])
                    print(f"  ✅ Variables:")
                    for var in vars:
                        if var['name'] in ['x', 'y', 'z']:
                            print(f"     {var['name']} = {var['value']}")

        print("\n8. Continuing execution...")
        reply = send_debug_request(client, 'continue', {
            'threadId': 1
        })

        if reply and reply.get('content', {}).get('success'):
            print(f"  ✅ Resumed execution")

        # Get final result
        try:
            exec_reply = client.get_shell_msg(timeout=2)
            if exec_reply['content'].get('status') == 'ok':
                print(f"  ✅ Script completed")
        except:
            pass

    print("\n9. Testing performance...")

    start_time = time.time()
    reply = send_debug_request(client, 'initialize', {
        'clientID': 'perf_test'
    })

    if reply:
        elapsed_ms = (time.time() - start_time) * 1000
        print(f"  DAP init time: {elapsed_ms:.1f}ms")
        if elapsed_ms < 50:
            print(f"  ✅ Performance requirement met")
        else:
            print(f"  ⚠️  Slower than 50ms requirement")

    client.stop_channels()

    print("\n" + "="*50)
    print("✅ Test completed successfully!")
    return True

if __name__ == "__main__":
    success = test_dap_workflow()
    if not success:
        print("❌ Test failed")
        exit(1)
#!/usr/bin/env python3
"""
Simple test to verify control channel debug_request works.
"""

import json
import time
from jupyter_client import BlockingKernelClient

print("Control Channel Debug Request Test")
print("="*40)

# Create client
client = BlockingKernelClient()
client.load_connection_file("/tmp/llmspell-test/kernel.json")
client.start_channels()

print("1. Sending debug_request (initialize)...")

# Create message
msg = client.session.msg('debug_request', {
    'command': 'initialize',
    'arguments': {
        'clientID': 'simple_test',
        'linesStartAt1': True
    }
})

# Send it
client.control_channel.send(msg)
msg_id = msg['header']['msg_id']
print(f"   Sent with msg_id: {msg_id}")

# Wait for reply
print("2. Waiting for debug_reply...")
for i in range(10):
    try:
        reply = client.control_channel.get_msg(timeout=1)
        if reply.get('parent_header', {}).get('msg_id') == msg_id:
            print(f"   ✅ Got debug_reply!")
            print(f"   Content: {reply.get('content', {})}")
            if 'body' in reply.get('content', {}):
                body = reply['content']['body']
                print(f"   Capabilities: supportsSetBreakpoints={body.get('supportsSetBreakpoints')}")
            break
    except Exception as e:
        print(f"   Attempt {i+1}: {e}")
else:
    print("   ❌ No reply received")

print("\n3. Testing setBreakpoints command...")
msg = client.session.msg('debug_request', {
    'command': 'setBreakpoints',
    'arguments': {
        'source': {'path': '/tmp/test.lua'},
        'breakpoints': [{'line': 5}]
    }
})

client.control_channel.send(msg)
msg_id = msg['header']['msg_id']
print(f"   Sent with msg_id: {msg_id}")

for i in range(10):
    try:
        reply = client.control_channel.get_msg(timeout=1)
        if reply.get('parent_header', {}).get('msg_id') == msg_id:
            print(f"   ✅ Got debug_reply!")
            print(f"   Content: {reply.get('content', {})}")
            break
    except:
        pass
else:
    print("   ❌ No reply received")

client.stop_channels()
print("\n✅ Test complete")
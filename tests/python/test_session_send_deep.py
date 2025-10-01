#!/usr/bin/env python3
"""
Deep trace of Session.send() to see actual ZMQ operations.
"""

import json
import zmq
from jupyter_client import BlockingKernelClient

# Monkey-patch zmq socket to trace sends
original_send_multipart = zmq.Socket.send_multipart

def traced_send_multipart(self, msg_parts, **kwargs):
    """Trace what's actually sent via ZMQ."""
    print(f"\n🔍 ZMQ send_multipart() called on {self.socket_type} socket:")
    print(f"   Number of parts: {len(msg_parts)}")
    for i, part in enumerate(msg_parts):
        if isinstance(part, bytes):
            if len(part) < 100:
                try:
                    decoded = part.decode('utf-8')
                    # Check if it's JSON
                    try:
                        parsed = json.loads(decoded)
                        print(f"   Part {i} (JSON): {list(parsed.keys()) if isinstance(parsed, dict) else decoded[:50]}")
                    except:
                        print(f"   Part {i} (string): {decoded[:50]}")
                except:
                    print(f"   Part {i} (bytes): {part[:50]}")
            else:
                print(f"   Part {i}: {len(part)} bytes")
        else:
            print(f"   Part {i} (non-bytes): {type(part)}")

    # Call original
    result = original_send_multipart(self, msg_parts, **kwargs)
    print(f"   ✅ send_multipart completed")
    return result

# Apply monkey patch
zmq.Socket.send_multipart = traced_send_multipart

print("="*60)
print("DEEP TRACE OF SESSION.SEND()")
print("="*60)

# Create client
client = BlockingKernelClient()
client.load_connection_file("/tmp/llmspell-test/kernel.json")
client.start_channels()

print("\n1. Sending debug_request via control channel...")
msg = client.session.msg('debug_request', {
    'command': 'initialize',
    'arguments': {'clientID': 'deep_trace_test'}
})

print(f"   Message type in dict: {msg.get('msg_type')}")
print(f"   Calling control_channel.send()...")
client.control_channel.send(msg)

print("\n2. Waiting for any ZMQ operations...")
import time
time.sleep(0.5)

print("\n3. For comparison, sending execute_request via shell...")
exec_id = client.execute('print("test")', silent=False)
print(f"   Execute msg_id: {exec_id}")

print("\n4. Checking for replies...")
try:
    reply = client.control_channel.get_msg(timeout=1)
    print(f"   ✅ Control channel got reply: {reply.get('msg_type')}")
except:
    print(f"   ❌ No control channel reply")

try:
    reply = client.get_shell_msg(timeout=1)
    print(f"   ✅ Shell channel got reply: {reply.get('content', {}).get('status')}")
except:
    print(f"   ❌ No shell reply")

client.stop_channels()

print("\n" + "="*60)
print("ANALYSIS:")
print("Compare the multipart messages sent for control vs shell")
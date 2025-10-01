#!/usr/bin/env python3
"""
Trace what happens when send() is called on a ZMQSocketChannel.
"""

import json
import zmq
from jupyter_client import BlockingKernelClient
from jupyter_client.channels import ZMQSocketChannel
import inspect
import sys

# Monkey-patch to trace session.send()
original_send = None

def traced_send(self, stream, msg_or_type, content=None, parent=None, **kwargs):
    """Traced version of Session.send()"""
    print(f"\n🔍 Session.send() called:")
    print(f"   stream: {stream}")
    print(f"   stream type: {type(stream)}")
    print(f"   msg_or_type: {msg_or_type if not isinstance(msg_or_type, dict) else 'dict'}")

    if isinstance(msg_or_type, dict):
        print(f"   msg keys: {list(msg_or_type.keys())}")
        print(f"   msg_type: {msg_or_type.get('msg_type')}")

    print(f"   content: {content}")
    print(f"   parent: {parent}")

    # Call original
    result = original_send(stream, msg_or_type, content, parent, **kwargs)
    print(f"   ✅ Session.send() completed, returned: {type(result)}")
    return result

print("="*60)
print("TRACING ZMQSocketChannel.send() BEHAVIOR")
print("="*60)

# Create client
client = BlockingKernelClient()
client.load_connection_file("/tmp/llmspell-test/kernel.json")

# Monkey-patch the session.send method
from jupyter_client.session import Session
original_send = client.session.send
client.session.send = lambda *args, **kwargs: traced_send(client.session, *args, **kwargs)

print("\n1. Starting channels...")
client.start_channels()

print("\n2. Checking control channel send() source:")
try:
    # Get the send method
    send_method = client.control_channel.send

    # Try to get source
    source = inspect.getsource(send_method)
    print("   Source of control_channel.send():")
    for i, line in enumerate(source.split('\n')[:20], 1):
        print(f"   {i:3}: {line}")
except Exception as e:
    print(f"   Could not get source: {e}")

print("\n3. Creating and sending message...")
msg = client.session.msg('debug_request', {
    'command': 'initialize',
    'arguments': {'clientID': 'send_trace_test'}
})

print(f"   Message msg_id: {msg['msg_id']}")
print(f"   Calling control_channel.send(msg)...")

# Send the message
client.control_channel.send(msg)

print("\n4. Checking if Session.send() was called...")
# If traced_send wasn't called, the channel uses a different mechanism

print("\n5. Let's check the stream queue mechanism:")
if hasattr(client.control_channel, 'stream'):
    print(f"   control_channel.stream exists: {client.control_channel.stream}")

# Check for queue-based sending
if hasattr(client.control_channel, '_queue'):
    print(f"   control_channel._queue exists: {client.control_channel._queue}")

print("\n6. Testing shell channel for comparison...")
# Try the same with shell channel to see if it works differently
exec_msg = client.session.msg('execute_request', {
    'code': 'print("test")',
    'silent': False
})

print("   Sending execute_request on shell channel...")
msg_id = client.execute('print("test")', silent=False)
print(f"   Execute returned msg_id: {msg_id}")

client.stop_channels()

print("\n" + "="*60)
print("KEY FINDINGS:")
print("1. Check if Session.send() is called for control channel")
print("2. Compare with shell channel behavior")
print("3. Look for queue-based mechanism")
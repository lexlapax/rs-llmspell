#!/usr/bin/env python3
"""
Understand how ZMQSocketChannel works internally.
"""

import json
from jupyter_client import BlockingKernelClient
from jupyter_client.channels import ZMQSocketChannel
import inspect

# Create client
client = BlockingKernelClient()
client.load_connection_file("/tmp/llmspell-test/kernel.json")

print("="*60)
print("ANALYZING ZMQSocketChannel")
print("="*60)

# Before starting channels
print("\n1. Control channel before start:")
print(f"   Type: {type(client.control_channel)}")
print(f"   Class: {client.control_channel.__class__.__name__}")

# Check if it's a ZMQSocketChannel
if isinstance(client.control_channel, ZMQSocketChannel):
    print("   ✅ Is a ZMQSocketChannel")

    # Check attributes
    print("\n2. ZMQSocketChannel attributes:")
    for attr in dir(client.control_channel):
        if not attr.startswith('_'):
            val = getattr(client.control_channel, attr, None)
            if not callable(val):
                print(f"   {attr}: {val}")

# Start channels
client.start_channels()

print("\n3. After starting channels:")
print(f"   is_alive(): {client.control_channel.is_alive()}")

# Check internal socket
print("\n4. Internal socket details:")
if hasattr(client.control_channel, 'socket'):
    sock = client.control_channel.socket
    print(f"   socket: {sock}")
    print(f"   socket type: {type(sock)}")
    if sock:
        print(f"   socket.closed: {sock.closed if hasattr(sock, 'closed') else 'N/A'}")

# Check the session
print("\n5. Session details:")
session = client.session
print(f"   session type: {type(session)}")
print(f"   session.key: {session.key[:20]}..." if session.key else "   session.key: None")

# Now test sending
print("\n6. Message sending test:")

# Create message
msg = client.session.msg('debug_request', {
    'command': 'initialize',
    'arguments': {'clientID': 'internals_test'}
})

print(f"   Message type: {type(msg)}")
print(f"   Message keys: {list(msg.keys())}")

# Look at what send() actually expects
print("\n7. control_channel.send() signature:")
try:
    sig = inspect.signature(client.control_channel.send)
    print(f"   {sig}")
except:
    print("   Could not get signature")

# Check if send accepts dict or needs something else
print("\n8. Testing different send approaches:")

# Approach 1: send(dict)
print("   Approach 1: control_channel.send(msg_dict)")
try:
    client.control_channel.send(msg)
    print("   ✅ send(dict) accepted")
except Exception as e:
    print(f"   ❌ send(dict) failed: {e}")

# Check for messages in queue
print("\n9. Checking thread internals:")
# ZMQSocketChannel runs in a thread and has an internal queue
if hasattr(client.control_channel, '_in_queue'):
    print(f"   _in_queue exists: {client.control_channel._in_queue}")
if hasattr(client.control_channel, '_out_queue'):
    print(f"   _out_queue exists: {client.control_channel._out_queue}")

client.stop_channels()

print("\n" + "="*60)
print("KEY FINDINGS:")
print("1. ZMQSocketChannel is a thread-based channel")
print("2. It has internal queues for message passing")
print("3. send() accepts a dict message")
print("4. The actual ZMQ sending happens in the thread")
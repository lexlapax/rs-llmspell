#!/usr/bin/env python3
"""
Debug script to understand how jupyter_client sends control channel messages.
"""

import json
import zmq
import time
from pathlib import Path
from jupyter_client import BlockingKernelClient

# First, let's see what methods the control channel has
print("Analyzing jupyter_client control channel...")

# Create a test client
client = BlockingKernelClient()
client.load_connection_file("/tmp/llmspell-test/kernel.json")
client.start_channels()

print(f"\nControl channel type: {type(client.control_channel)}")
print(f"Control channel methods: {[m for m in dir(client.control_channel) if not m.startswith('_')]}")

# Check if control channel has a socket
if hasattr(client.control_channel, 'socket'):
    print(f"Control channel socket: {client.control_channel.socket}")
    print(f"Socket type: {type(client.control_channel.socket)}")

# Try different ways to send a message
msg = client.session.msg('debug_request', {
    'command': 'initialize',
    'arguments': {}
})

print(f"\nMessage structure from session.msg():")
print(f"  Keys: {msg.keys()}")
print(f"  Header: {msg['header']}")

# Method 1: Try control_channel.send()
try:
    print("\nMethod 1: control_channel.send(msg)")
    client.control_channel.send(msg)
    print("  ✓ send() succeeded")
except Exception as e:
    print(f"  ✗ send() failed: {e}")

# Method 2: Try getting the socket directly
try:
    print("\nMethod 2: Access socket directly")
    # Control channels in jupyter_client are ZMQSocketChannel objects
    # They run in a thread and have an internal socket

    # The actual socket is accessed differently
    # Let's check the session's send method
    print(f"  Session type: {type(client.session)}")
    print(f"  Session send signature: {client.session.send.__doc__}")

except Exception as e:
    print(f"  ✗ Failed: {e}")

# Method 3: Use session.send with the control channel
try:
    print("\nMethod 3: session.send(control_channel.socket, msg_type, content)")
    # This is how jupyter_client usually sends messages
    # But control_channel.socket might not be directly accessible

    # Actually, let's check what happens when we call execute
    print("  Checking how execute() sends messages...")

    # Look at the implementation
    import inspect
    print(f"  Execute method source file: {inspect.getfile(client.execute)}")

except Exception as e:
    print(f"  ✗ Failed: {e}")

client.stop_channels()

print("\n" + "="*60)
print("Key findings:")
print("1. Control channel is a ZMQSocketChannel (runs in thread)")
print("2. send() method exists but may not format correctly")
print("3. Need to investigate ZMQSocketChannel.send() implementation")
#!/usr/bin/env python3
"""
Create a custom channel to capture exact send operations.
"""

import json
import zmq
from jupyter_client import BlockingKernelClient
from jupyter_client.channels import ZMQSocketChannel
import threading
import queue

class DebugZMQSocketChannel(ZMQSocketChannel):
    """Custom channel that logs all send operations."""

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.sent_messages = []

    def _send(self, msg):
        """Override internal send to capture messages."""
        print(f"\n🔍 DebugChannel._send() called")
        print(f"   Message type: {type(msg)}")
        print(f"   Message keys: {list(msg.keys()) if isinstance(msg, dict) else 'not a dict'}")

        # Log the message
        self.sent_messages.append(msg)

        # Call the parent implementation
        super()._send(msg)
        print(f"   ✅ Parent _send() completed")

    def _run_thread(self):
        """Override thread runner to add logging."""
        print("🔍 DebugChannel thread started")
        super()._run_thread()

class DebugBlockingKernelClient(BlockingKernelClient):
    """Client that uses debug channels."""

    def _create_default_control_channel(self):
        """Create a debug control channel."""
        print("Creating debug control channel...")
        return DebugZMQSocketChannel(
            self.context,
            self.session,
            f"tcp://{self._ip}:{self._control_port}"
        )

# Test with debug client
print("="*60)
print("TESTING WITH DEBUG CHANNEL")
print("="*60)

# Load connection info
with open("/tmp/llmspell-test/kernel.json") as f:
    conn_info = json.load(f)

# Create debug client
client = DebugBlockingKernelClient()
client.load_connection_file("/tmp/llmspell-test/kernel.json")

print("\n1. Starting channels...")
client.start_channels()
print("   ✅ Channels started")

print("\n2. Creating debug_request message...")
msg = client.session.msg('debug_request', {
    'command': 'initialize',
    'arguments': {'clientID': 'debug_channel_test'}
})
print(f"   Message created with msg_id: {msg['msg_id']}")

print("\n3. Sending via control channel...")
client.control_channel.send(msg)
print("   ✅ send() returned")

print("\n4. Checking sent messages...")
if hasattr(client.control_channel, 'sent_messages'):
    print(f"   Sent {len(client.control_channel.sent_messages)} messages")
    for i, m in enumerate(client.control_channel.sent_messages):
        print(f"   Message {i}: {list(m.keys()) if isinstance(m, dict) else type(m)}")

print("\n5. Waiting for reply...")
try:
    reply = client.control_channel.get_msg(timeout=2)
    print(f"   ✅ Got reply: {reply.get('msg_type')}")
except Exception as e:
    print(f"   ❌ No reply: {e}")

print("\n6. Checking control channel internals...")
# Try to access the thread's internal state
if hasattr(client.control_channel, '_thread'):
    thread = client.control_channel._thread
    print(f"   Thread alive: {thread.is_alive() if thread else 'No thread'}")

if hasattr(client.control_channel, 'socket'):
    sock = client.control_channel.socket
    print(f"   Socket state: {sock}")
    print(f"   Socket type: {sock.socket_type if hasattr(sock, 'socket_type') else 'unknown'}")

client.stop_channels()

print("\n" + "="*60)
print("ANALYSIS:")
print("The debug channel should show us exactly what's being sent")
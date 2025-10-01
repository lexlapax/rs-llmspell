#!/usr/bin/env python3
"""
Compare exact message format between raw ZeroMQ and jupyter_client.
This will help identify why control channel messages from jupyter_client aren't received.
"""

import json
import zmq
import time
import hmac
import hashlib
import uuid
from pathlib import Path
from datetime import datetime
from jupyter_client import BlockingKernelClient
import threading

# Intercept messages with a proxy
class MessageCapture:
    def __init__(self):
        self.messages = []
        self.context = zmq.Context()

    def create_proxy(self, frontend_port, backend_port):
        """Create a proxy to capture messages between client and kernel."""
        # Frontend socket (clients connect here)
        frontend = self.context.socket(zmq.ROUTER)
        frontend.bind(f"tcp://127.0.0.1:{frontend_port}")

        # Backend socket (connects to real kernel)
        backend = self.context.socket(zmq.DEALER)
        backend.connect(f"tcp://127.0.0.1:{backend_port}")

        # Proxy and capture
        poller = zmq.Poller()
        poller.register(frontend, zmq.POLLIN)
        poller.register(backend, zmq.POLLIN)

        def run_proxy():
            while True:
                socks = dict(poller.poll(100))

                if frontend in socks:
                    # Message from client to kernel
                    msg = frontend.recv_multipart()
                    print(f"\n📤 Client->Kernel ({len(msg)} parts):")
                    for i, part in enumerate(msg):
                        if len(part) < 100:
                            try:
                                # Try to decode as string
                                decoded = part.decode('utf-8')
                                print(f"  Part {i} (string): {decoded[:100]}")
                            except:
                                print(f"  Part {i} (bytes): {part[:100]}")
                        else:
                            print(f"  Part {i}: {len(part)} bytes")
                    self.messages.append(('client->kernel', msg))
                    backend.send_multipart(msg[1:])  # Strip routing ID for DEALER

                if backend in socks:
                    # Message from kernel to client
                    msg = backend.recv_multipart()
                    print(f"\n📥 Kernel->Client ({len(msg)} parts)")
                    # Need to add routing ID for ROUTER
                    # This is complex - skipping for now

        thread = threading.Thread(target=run_proxy)
        thread.daemon = True
        thread.start()

        return frontend_port

def test_raw_zmq_format():
    """Show exact message format from raw ZeroMQ."""
    print("\n" + "="*60)
    print("RAW ZEROMQ MESSAGE FORMAT")
    print("="*60)

    # Load connection info
    with open("/tmp/llmspell-test/kernel.json") as f:
        conn_info = json.load(f)

    # Create socket
    context = zmq.Context()
    socket = context.socket(zmq.DEALER)
    socket.connect(f"tcp://127.0.0.1:{conn_info['control_port']}")

    # Build debug_request message
    session = str(uuid.uuid4())
    msg_id = str(uuid.uuid4())

    header = {
        "msg_id": msg_id,
        "msg_type": "debug_request",
        "session": session,
        "username": "test",
        "date": datetime.utcnow().isoformat() + 'Z',
        "version": "5.3"
    }

    parent_header = {}
    metadata = {}
    content = {
        "command": "initialize",
        "arguments": {
            "clientID": "message_comparison",
            "linesStartAt1": True
        }
    }

    # Serialize parts
    header_bytes = json.dumps(header).encode('utf-8')
    parent_header_bytes = json.dumps(parent_header).encode('utf-8')
    metadata_bytes = json.dumps(metadata).encode('utf-8')
    content_bytes = json.dumps(content).encode('utf-8')

    # Create HMAC signature
    h = hmac.new(
        conn_info['key'].encode('utf-8'),
        digestmod=hashlib.sha256
    )
    h.update(header_bytes)
    h.update(parent_header_bytes)
    h.update(metadata_bytes)
    h.update(content_bytes)
    signature = h.hexdigest()

    # Build multipart message
    message = [
        b'<IDS|MSG>',
        signature.encode('utf-8'),
        header_bytes,
        parent_header_bytes,
        metadata_bytes,
        content_bytes
    ]

    print(f"Sending {len(message)} parts:")
    for i, part in enumerate(message):
        if len(part) < 100:
            try:
                print(f"  Part {i}: {part.decode('utf-8')}")
            except:
                print(f"  Part {i}: {part}")
        else:
            print(f"  Part {i}: {len(part)} bytes")

    socket.send_multipart(message)

    # Get reply
    try:
        reply = socket.recv_multipart()
        print(f"\n✅ Got reply with {len(reply)} parts")
        return True
    except:
        print("\n❌ No reply received")
        return False

def test_jupyter_client_format():
    """Show exact message format from jupyter_client."""
    print("\n" + "="*60)
    print("JUPYTER_CLIENT MESSAGE FORMAT")
    print("="*60)

    # Create client
    client = BlockingKernelClient()
    client.load_connection_file("/tmp/llmspell-test/kernel.json")

    # Let's examine the control channel before starting
    print(f"Control channel type: {type(client.control_channel)}")
    print(f"Control channel class: {client.control_channel.__class__.__name__}")

    # Start channels
    client.start_channels()

    # Build message using session
    msg = client.session.msg('debug_request', {
        'command': 'initialize',
        'arguments': {
            'clientID': 'jupyter_comparison'
        }
    })

    print(f"\nMessage created by session.msg():")
    print(f"  Type: {type(msg)}")
    print(f"  Keys: {msg.keys()}")

    # Now let's see what control_channel.send() actually does
    # First, let's check if it has _send method
    if hasattr(client.control_channel, '_send'):
        print(f"  control_channel has _send method")

    # Check the actual socket
    if hasattr(client.control_channel, 'socket'):
        print(f"  control_channel.socket: {client.control_channel.socket}")

    # Try to trace what send() does
    import inspect
    print(f"\ncontrol_channel.send method:")
    try:
        source = inspect.getsource(client.control_channel.send)
        print("  Source code:")
        for line in source.split('\n')[:10]:
            print(f"    {line}")
    except:
        print("  Could not get source")

    # Actually send it
    print("\nSending via control_channel.send()...")
    client.control_channel.send(msg)

    # Try to get reply
    try:
        reply = client.control_channel.get_msg(timeout=2)
        print(f"✅ Got reply: {reply.get('msg_type')}")
        return True
    except:
        print("❌ No reply received")
        return False

def test_with_proxy():
    """Use a proxy to capture actual bytes sent."""
    print("\n" + "="*60)
    print("CAPTURING WITH PROXY")
    print("="*60)

    # Load original connection info
    with open("/tmp/llmspell-test/kernel.json") as f:
        conn_info = json.load(f)

    # Create proxy
    capture = MessageCapture()
    proxy_port = 9999
    real_port = conn_info['control_port']

    # Start proxy
    print(f"Starting proxy: {proxy_port} -> {real_port}")
    capture.create_proxy(proxy_port, real_port)

    # Modify connection file for client
    proxy_conn = conn_info.copy()
    proxy_conn['control_port'] = proxy_port

    with open("/tmp/proxy-kernel.json", 'w') as f:
        json.dump(proxy_conn, f)

    time.sleep(1)

    # Test with jupyter_client through proxy
    print("\nTesting jupyter_client through proxy...")
    client = BlockingKernelClient()
    client.load_connection_file("/tmp/proxy-kernel.json")
    client.start_channels()

    msg = client.session.msg('debug_request', {
        'command': 'initialize',
        'arguments': {'clientID': 'proxy_test'}
    })

    client.control_channel.send(msg)
    time.sleep(2)

    # Check captured messages
    print(f"\nCaptured {len(capture.messages)} messages")

    client.stop_channels()

if __name__ == "__main__":
    print("Message Format Comparison Test")
    print("==============================")

    # Test 1: Raw ZeroMQ (works)
    raw_works = test_raw_zmq_format()

    # Test 2: jupyter_client (doesn't work)
    jupyter_works = test_jupyter_client_format()

    # Test 3: Capture with proxy
    test_with_proxy()

    print("\n" + "="*60)
    print("RESULTS:")
    print(f"  Raw ZeroMQ:     {'✅ WORKS' if raw_works else '❌ FAILS'}")
    print(f"  jupyter_client: {'✅ WORKS' if jupyter_works else '❌ FAILS'}")
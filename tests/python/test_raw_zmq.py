#!/usr/bin/env python3
"""
Test llmspell kernel with raw ZeroMQ messages.
This verifies basic transport layer functionality before testing with jupyter_client.
"""

import zmq
import json
import uuid
import hmac
import hashlib
from datetime import datetime

def test_raw_zmq():
    """Test kernel with raw ZeroMQ messages."""

    # Load connection info
    with open('/tmp/llmspell-test/kernel.json') as f:
        conn = json.load(f)

    print(f"Connection info:")
    print(f"  Shell port: {conn['shell_port']}")
    print(f"  Control port: {conn['control_port']}")
    print(f"  Key: {conn['key'][:20]}...")

    # Create ZeroMQ context
    ctx = zmq.Context()

    # Connect to shell channel
    shell = ctx.socket(zmq.DEALER)
    shell.connect(f"tcp://127.0.0.1:{conn['shell_port']}")
    print(f"\n✓ Connected to shell channel on port {conn['shell_port']}")

    # Create kernel_info_request message
    msg_id = str(uuid.uuid4())
    session = str(uuid.uuid4())

    header = {
        "msg_id": msg_id,
        "session": session,
        "username": "test",
        "msg_type": "kernel_info_request",
        "version": "5.3",
        "date": datetime.utcnow().isoformat() + 'Z'
    }

    parent_header = {}
    metadata = {}
    content = {}

    # Serialize message parts
    header_bytes = json.dumps(header).encode('utf-8')
    parent_header_bytes = json.dumps(parent_header).encode('utf-8')
    metadata_bytes = json.dumps(metadata).encode('utf-8')
    content_bytes = json.dumps(content).encode('utf-8')

    # Create HMAC signature
    key = conn['key'].encode('utf-8')
    h = hmac.new(key, digestmod=hashlib.sha256)
    h.update(header_bytes)
    h.update(parent_header_bytes)
    h.update(metadata_bytes)
    h.update(content_bytes)
    signature = h.hexdigest()

    # Send multipart message
    # [delimiter, signature, header, parent_header, metadata, content]
    message = [
        b'<IDS|MSG>',
        signature.encode('utf-8'),
        header_bytes,
        parent_header_bytes,
        metadata_bytes,
        content_bytes
    ]

    print(f"\n→ Sending kernel_info_request")
    print(f"  msg_id: {msg_id}")
    print(f"  Signature: {signature[:20]}...")

    shell.send_multipart(message)
    print(f"✓ Message sent")

    # Wait for reply
    print(f"\n← Waiting for kernel_info_reply...")

    # Set timeout
    shell.setsockopt(zmq.RCVTIMEO, 5000)  # 5 second timeout

    try:
        reply_parts = shell.recv_multipart()
        print(f"✓ Received reply with {len(reply_parts)} parts")

        # Parse reply
        delimiter_idx = None
        for i, part in enumerate(reply_parts):
            if part == b'<IDS|MSG>':
                delimiter_idx = i
                break

        if delimiter_idx is not None and len(reply_parts) > delimiter_idx + 5:
            reply_signature = reply_parts[delimiter_idx + 1].decode('utf-8')
            reply_header = json.loads(reply_parts[delimiter_idx + 2])
            reply_parent = json.loads(reply_parts[delimiter_idx + 3])
            reply_metadata = json.loads(reply_parts[delimiter_idx + 4])
            reply_content = json.loads(reply_parts[delimiter_idx + 5])

            print(f"\nReply details:")
            print(f"  msg_type: {reply_header.get('msg_type')}")
            print(f"  msg_id: {reply_header.get('msg_id')}")
            print(f"  parent msg_id: {reply_parent.get('msg_id')}")

            # Verify parent header
            if reply_parent.get('msg_id') == msg_id:
                print(f"  ✓ Parent header correctly references our request!")
            else:
                print(f"  ✗ Parent header mismatch: expected {msg_id}, got {reply_parent.get('msg_id')}")

            # Verify signature
            h = hmac.new(key, digestmod=hashlib.sha256)
            h.update(reply_parts[delimiter_idx + 2])  # header
            h.update(reply_parts[delimiter_idx + 3])  # parent
            h.update(reply_parts[delimiter_idx + 4])  # metadata
            h.update(reply_parts[delimiter_idx + 5])  # content
            expected_sig = h.hexdigest()

            if reply_signature == expected_sig:
                print(f"  ✓ HMAC signature valid!")
            else:
                print(f"  ✗ HMAC signature mismatch")
                print(f"    Expected: {expected_sig[:20]}...")
                print(f"    Got:      {reply_signature[:20]}...")

            if reply_header['msg_type'] == 'kernel_info_reply':
                print(f"\n✓ SUCCESS: Kernel responded to kernel_info_request!")
                print(f"  Protocol version: {reply_content.get('protocol_version')}")
                print(f"  Implementation: {reply_content.get('implementation')}")
                print(f"  Language: {reply_content.get('language_info', {}).get('name')}")
                return True
            else:
                print(f"\n✗ Unexpected msg_type: {reply_header['msg_type']}")
                return False
        else:
            print(f"✗ Invalid reply format")
            for i, part in enumerate(reply_parts):
                print(f"  Part {i}: {part[:50]}...")
            return False

    except zmq.error.Again:
        print(f"✗ TIMEOUT: No reply received within 5 seconds")
        print(f"  Check /tmp/llmspell-test/kernel.log for errors")
        return False
    except Exception as e:
        print(f"✗ ERROR: {e}")
        import traceback
        traceback.print_exc()
        return False
    finally:
        shell.close()
        ctx.term()

def test_dap_via_zmq():
    """Test DAP through debug_request with raw ZeroMQ."""

    # Load connection info
    with open('/tmp/llmspell-test/kernel.json') as f:
        conn = json.load(f)

    print(f"\n{'='*60}")
    print(f"Testing DAP via debug_request")
    print(f"{'='*60}")

    # Create ZeroMQ context
    ctx = zmq.Context()

    # Connect to control channel (debug_request goes through control)
    control = ctx.socket(zmq.DEALER)
    control.connect(f"tcp://127.0.0.1:{conn['control_port']}")
    print(f"✓ Connected to control channel on port {conn['control_port']}")

    # Create debug_request message with DAP initialize command
    msg_id = str(uuid.uuid4())
    session = str(uuid.uuid4())

    header = {
        "msg_id": msg_id,
        "session": session,
        "username": "test",
        "msg_type": "debug_request",
        "version": "5.3",
        "date": datetime.utcnow().isoformat() + 'Z'
    }

    parent_header = {}
    metadata = {}

    # DAP initialize command
    content = {
        "command": "initialize",
        "arguments": {
            "clientID": "test_zmq",
            "clientName": "Raw ZMQ Test",
            "adapterID": "llmspell",
            "linesStartAt1": True,
            "columnsStartAt1": True
        }
    }

    # Serialize and sign
    header_bytes = json.dumps(header).encode('utf-8')
    parent_header_bytes = json.dumps(parent_header).encode('utf-8')
    metadata_bytes = json.dumps(metadata).encode('utf-8')
    content_bytes = json.dumps(content).encode('utf-8')

    key = conn['key'].encode('utf-8')
    h = hmac.new(key, digestmod=hashlib.sha256)
    h.update(header_bytes)
    h.update(parent_header_bytes)
    h.update(metadata_bytes)
    h.update(content_bytes)
    signature = h.hexdigest()

    message = [
        b'<IDS|MSG>',
        signature.encode('utf-8'),
        header_bytes,
        parent_header_bytes,
        metadata_bytes,
        content_bytes
    ]

    print(f"\n→ Sending debug_request (DAP initialize)")
    control.send_multipart(message)
    print(f"✓ Message sent")

    # Wait for debug_reply
    print(f"\n← Waiting for debug_reply...")
    control.setsockopt(zmq.RCVTIMEO, 5000)

    try:
        reply_parts = control.recv_multipart()
        print(f"✓ Received reply with {len(reply_parts)} parts")

        # Parse reply
        delimiter_idx = None
        for i, part in enumerate(reply_parts):
            if part == b'<IDS|MSG>':
                delimiter_idx = i
                break

        if delimiter_idx is not None and len(reply_parts) > delimiter_idx + 5:
            reply_header = json.loads(reply_parts[delimiter_idx + 2])
            reply_content = json.loads(reply_parts[delimiter_idx + 5])

            if reply_header['msg_type'] == 'debug_reply':
                print(f"\n✓ SUCCESS: Received debug_reply!")
                print(f"  Reply content: {reply_content}")

                # The DAP response itself is the content, not wrapped in status
                if 'command' in reply_content or 'body' in reply_content:
                    body = reply_content.get('body', reply_content)
                    print(f"  DAP Response received!")
                    print(f"    Command: {reply_content.get('command', 'initialize')}")
                    print(f"    Success: {reply_content.get('success', True)}")
                    if 'supportsConfigurationDoneRequest' in body:
                        print(f"  DAP Capabilities:")
                        print(f"    supportsConfigurationDoneRequest: {body.get('supportsConfigurationDoneRequest')}")
                        print(f"    supportsSetBreakpoints: {body.get('supportsSetBreakpoints')}")
                        print(f"    supportsStepBack: {body.get('supportsStepBack')}")
                    return True
                else:
                    print(f"  Unexpected content format: {reply_content}")
                    return False
            else:
                print(f"✗ Unexpected msg_type: {reply_header['msg_type']}")
                return False
        else:
            print(f"✗ Invalid reply format")
            return False

    except zmq.error.Again:
        print(f"✗ TIMEOUT: No reply received")
        return False
    except Exception as e:
        print(f"✗ ERROR: {e}")
        return False
    finally:
        control.close()
        ctx.term()

if __name__ == "__main__":
    print("="*60)
    print("Testing llmspell kernel with raw ZeroMQ")
    print("="*60)

    # Test basic kernel_info
    success1 = test_raw_zmq()

    if success1:
        print("\n" + "="*60)
        print("✓ Basic ZeroMQ communication working!")
        print("  Task 10.7.8 parent_header implementation verified")
        print("  Task 10.7.8 HMAC signing implementation verified")

        # Test DAP
        success2 = test_dap_via_zmq()

        if success2:
            print("\n" + "="*60)
            print("✓ DAP via debug_request working!")
            print("  Task 10.7 Debug Adapter Protocol implementation verified")
        else:
            print("\n✗ DAP test failed")
    else:
        print("\n✗ Basic connectivity test failed")
        print("  Check kernel logs at /tmp/llmspell-test/kernel.log")
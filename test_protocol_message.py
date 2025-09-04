#!/usr/bin/env python3

import zmq
import json
import uuid
import hmac
import hashlib
from datetime import datetime

def sign_message(key, *message_parts):
    """Sign message using HMAC-SHA256"""
    h = hmac.new(key.encode('utf-8'), digestmod=hashlib.sha256)
    for part in message_parts:
        h.update(part)
    return h.hexdigest()

def send_message(socket, msg_type, content, key, session_id="test"):
    """Send a Jupyter message"""
    header = {
        "msg_id": str(uuid.uuid4()),
        "msg_type": msg_type,
        "username": "test",
        "session": session_id,
        "date": datetime.utcnow().isoformat() + "Z",
        "version": "5.3"
    }
    
    # Encode message parts
    header_bytes = json.dumps(header).encode('utf-8')
    parent_header_bytes = b''  # Empty for initial request
    metadata_bytes = b'{}'
    content_bytes = json.dumps(content).encode('utf-8')
    
    # Sign the message
    signature = sign_message(key, header_bytes, parent_header_bytes, metadata_bytes, content_bytes)
    
    # Send multipart message with proper delimiter
    # Format: [identity, delimiter, signature, header, parent_header, metadata, content]
    socket.send_multipart([
        b'',  # Empty identity
        b'<IDS|MSG>',  # Delimiter
        signature.encode('utf-8'),
        header_bytes,
        parent_header_bytes,
        metadata_bytes,
        content_bytes
    ])

def recv_message(socket):
    """Receive a Jupyter message"""
    parts = socket.recv_multipart()
    
    # Find delimiter
    delim_idx = None
    for i, part in enumerate(parts):
        if part == b'<IDS|MSG>':
            delim_idx = i
            break
    
    if delim_idx is not None and delim_idx + 5 < len(parts):
        # Extract parts after delimiter
        signature = parts[delim_idx + 1]
        header_bytes = parts[delim_idx + 2]
        parent_header_bytes = parts[delim_idx + 3] 
        metadata_bytes = parts[delim_idx + 4]
        content_bytes = parts[delim_idx + 5]
        
        header = json.loads(header_bytes.decode('utf-8'))
        content = json.loads(content_bytes.decode('utf-8'))
        return header, content
    
    return None, None

def test_kernel_connection():
    """Test kernel connection with kernel_info and execute requests"""
    # Read connection file
    connection_file = "/var/folders/nc/r_scv0hd78x07x908ymg5mk80000gn/T/kernel-e87a6bfc-17bb-4e88-a30f-9173e1a2aeb9.json"
    
    with open(connection_file, 'r') as f:
        conn_info = json.load(f)
    
    print(f"Connection info: {conn_info}")
    
    context = zmq.Context()
    
    # Connect to shell channel
    shell = context.socket(zmq.REQ)
    shell.connect(f"tcp://{conn_info['ip']}:{conn_info['shell_port']}")
    
    # Test kernel_info_request
    print("Sending kernel_info_request...")
    send_message(shell, "kernel_info_request", {}, conn_info['key'])
    
    header, content = recv_message(shell)
    if header:
        print(f"Received: {header['msg_type']}")
        print(f"Content: {json.dumps(content, indent=2)}")
    else:
        print("No response received")
    
    shell.close()
    context.term()

if __name__ == "__main__":
    test_kernel_connection()
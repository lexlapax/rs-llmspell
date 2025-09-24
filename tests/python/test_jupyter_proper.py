#!/usr/bin/env python3
"""
Test proper usage of jupyter_client with llmspell kernel.
"""

import json
import logging
from jupyter_client import BlockingKernelClient

logging.basicConfig(level=logging.DEBUG, format='%(levelname)s:%(name)s:%(message)s')
logger = logging.getLogger(__name__)

def test_with_explicit_load():
    """Test loading connection file explicitly."""

    connection_file = "/tmp/llmspell-test/kernel.json"
    logger.info(f"Testing with explicit load_connection_file")

    # Create client WITHOUT passing connection_file to constructor
    client = BlockingKernelClient()

    # Load connection file explicitly
    client.load_connection_file(connection_file)

    logger.info(f"After load_connection_file:")
    logger.info(f"  shell_port: {client.shell_port}")
    logger.info(f"  control_port: {client.control_port}")
    logger.info(f"  iopub_port: {client.iopub_port}")
    logger.info(f"  hb_port: {client.hb_port}")
    logger.info(f"  ip: {client.ip}")

    if client.shell_port == 0:
        logger.error("Ports still 0 after load_connection_file")
        return False

    # Now start channels
    client.start_channels()

    try:
        # Send kernel_info_request
        logger.info("Sending kernel_info_request...")
        msg_id = client.kernel_info()

        # Wait for reply
        logger.info("Waiting for reply...")
        reply = client.get_shell_msg(timeout=5)

        if reply['msg_type'] == 'kernel_info_reply':
            logger.info("✓ SUCCESS: Kernel responded!")
            logger.info(f"  Protocol: {reply['content'].get('protocol_version')}")
            return True
        else:
            logger.error(f"Unexpected reply: {reply['msg_type']}")
            return False

    except Exception as e:
        logger.error(f"Error: {e}")
        return False
    finally:
        client.stop_channels()

def test_with_kernel_manager():
    """Test using KernelManager instead of direct client."""
    from jupyter_client import KernelManager

    connection_file = "/tmp/llmspell-test/kernel.json"
    logger.info(f"Testing with KernelManager")

    # Create KernelManager
    km = KernelManager(connection_file=connection_file)
    km.load_connection_file()

    logger.info(f"KernelManager loaded:")
    logger.info(f"  shell_port: {km.shell_port}")
    logger.info(f"  control_port: {km.control_port}")

    # Get a client from the manager
    client = km.client()
    client.start_channels()

    try:
        # Send kernel_info_request
        logger.info("Sending kernel_info_request via KernelManager...")
        msg_id = client.kernel_info()

        # Wait for reply
        reply = client.get_shell_msg(timeout=5)

        if reply['msg_type'] == 'kernel_info_reply':
            logger.info("✓ SUCCESS with KernelManager!")
            return True
        else:
            logger.error(f"Unexpected reply: {reply['msg_type']}")
            return False

    except Exception as e:
        logger.error(f"Error: {e}")
        return False
    finally:
        client.stop_channels()

def test_manual_connection_info():
    """Test by manually setting connection info."""

    # Load connection info manually
    with open("/tmp/llmspell-test/kernel.json") as f:
        conn_info = json.load(f)

    logger.info("Testing with manual connection info")

    # Create client and manually set all parameters
    client = BlockingKernelClient()
    client.ip = conn_info['ip']
    client.shell_port = conn_info['shell_port']
    client.iopub_port = conn_info['iopub_port']
    client.stdin_port = conn_info['stdin_port']
    client.control_port = conn_info['control_port']
    client.hb_port = conn_info['hb_port']
    client.session.key = conn_info['key'].encode()

    logger.info(f"Manually set:")
    logger.info(f"  shell_port: {client.shell_port}")
    logger.info(f"  control_port: {client.control_port}")

    # Start channels
    client.start_channels()

    try:
        # Send kernel_info_request
        logger.info("Sending kernel_info_request...")
        msg_id = client.kernel_info()

        # Wait for reply
        reply = client.get_shell_msg(timeout=5)

        if reply['msg_type'] == 'kernel_info_reply':
            logger.info("✓ SUCCESS with manual connection info!")
            return True
        else:
            logger.error(f"Unexpected reply: {reply['msg_type']}")
            return False

    except Exception as e:
        logger.error(f"Error: {e}")
        return False
    finally:
        client.stop_channels()

if __name__ == "__main__":
    print("="*60)
    print("Testing proper jupyter_client usage")
    print("="*60)

    # Test 1: Explicit load
    print("\n1. Testing explicit load_connection_file:")
    success1 = test_with_explicit_load()

    # Test 2: KernelManager
    print("\n2. Testing with KernelManager:")
    success2 = test_with_kernel_manager()

    # Test 3: Manual
    print("\n3. Testing with manual connection info:")
    success3 = test_manual_connection_info()

    print("\n" + "="*60)
    print("Results:")
    print(f"  Explicit load: {'✓ PASS' if success1 else '✗ FAIL'}")
    print(f"  KernelManager: {'✓ PASS' if success2 else '✗ FAIL'}")
    print(f"  Manual setup:  {'✓ PASS' if success3 else '✗ FAIL'}")

    if success1 or success2 or success3:
        print("\n✓ At least one method works - we know how to use jupyter_client correctly!")
        print("  Update TODO.md to remove 'jupyter_client has a bug' claim")
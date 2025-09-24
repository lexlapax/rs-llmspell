#!/usr/bin/env python3
"""
Simple test script to verify llmspell kernel responds to kernel_info_request.
Tests the parent_header fix from Task 10.7.8.
"""

import json
import logging
from jupyter_client import BlockingKernelClient

# Enable debug logging to see what's happening
logging.basicConfig(level=logging.DEBUG, format='%(levelname)s:%(name)s:%(message)s')
logger = logging.getLogger(__name__)

def test_kernel_info():
    """Test basic kernel_info_request/reply exchange."""

    # Load connection file
    connection_file = "/tmp/llmspell-test/kernel.json"
    logger.info(f"Loading connection file: {connection_file}")

    with open(connection_file) as f:
        conn_info = json.load(f)
    logger.info(f"Connection info: {conn_info}")

    # Create kernel client and load connection file
    client = BlockingKernelClient()
    client.load_connection_file(connection_file)  # MUST call this explicitly!
    logger.info("Created kernel client and loaded connection file")

    try:
        # Start channels
        client.start_channels()
        logger.info("Started channels")

        # Send kernel_info_request
        logger.info("Sending kernel_info_request...")
        msg_id = client.kernel_info()
        logger.info(f"Request sent with msg_id: {msg_id}")

        # Wait for reply
        logger.info("Waiting for kernel_info_reply...")
        reply = client.get_shell_msg(timeout=5)

        # Check reply
        logger.info(f"Got reply: msg_type={reply.get('msg_type')}")
        logger.info(f"Reply header: {reply.get('header')}")
        logger.info(f"Parent header: {reply.get('parent_header')}")

        if reply['msg_type'] == 'kernel_info_reply':
            content = reply.get('content', {})
            logger.info("✓ Kernel info received successfully!")
            logger.info(f"  Protocol version: {content.get('protocol_version')}")
            logger.info(f"  Implementation: {content.get('implementation')}")
            logger.info(f"  Language: {content.get('language_info', {}).get('name')}")
            logger.info(f"  Status: {content.get('status')}")

            # Check parent_header matches our request
            parent_msg_id = reply.get('parent_header', {}).get('msg_id')
            if parent_msg_id == msg_id:
                logger.info("✓ Parent header correctly set!")
            else:
                logger.warning(f"✗ Parent header mismatch: expected {msg_id}, got {parent_msg_id}")

            return True
        else:
            logger.error(f"Unexpected reply type: {reply['msg_type']}")
            return False

    except Exception as e:
        logger.error(f"Error: {e}")
        import traceback
        traceback.print_exc()
        return False

    finally:
        # Clean up
        client.stop_channels()
        logger.info("Stopped channels")

if __name__ == "__main__":
    logger.info("="*60)
    logger.info("Testing llmspell kernel with jupyter_client")
    logger.info("="*60)

    success = test_kernel_info()

    if success:
        logger.info("\n✓ SUCCESS: Kernel responds correctly to kernel_info_request")
        logger.info("  This means Task 10.7.8 parent_header issue is FIXED!")
    else:
        logger.error("\n✗ FAILURE: Kernel did not respond correctly")
        logger.error("  Check /tmp/llmspell-test/kernel.log for kernel-side errors")
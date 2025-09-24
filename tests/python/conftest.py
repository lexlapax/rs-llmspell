"""
Pytest configuration and fixtures for llmspell Jupyter integration tests.

This module provides session and test-scoped fixtures for managing the llmspell
kernel daemon lifecycle and creating Jupyter clients for testing.
"""

import pytest
import subprocess
import tempfile
import json
import time
import os
import signal
import atexit
from pathlib import Path
from jupyter_client import BlockingKernelClient
import logging

# Configure logging for debugging test issues
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Track all spawned processes for cleanup
_spawned_processes = []

def cleanup_processes():
    """Ensure all spawned processes are terminated on exit."""
    for proc in _spawned_processes:
        if proc.poll() is None:
            logger.info(f"Cleaning up process {proc.pid}")
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait()

atexit.register(cleanup_processes)


@pytest.fixture(scope="session")
def build_llmspell():
    """Build llmspell once per test session."""
    binary_path = Path("./target/debug/llmspell")

    # Check if binary already exists
    if binary_path.exists():
        logger.info(f"Using existing llmspell binary at {binary_path}")
        return binary_path

    logger.info("Building llmspell...")
    result = subprocess.run(
        ["cargo", "build", "--all-targets", "--all-features"],
        capture_output=True,
        text=True,
        check=False,
        timeout=300  # 5 minute timeout for build
    )
    if result.returncode != 0:
        logger.error(f"Build failed:\n{result.stderr}")
        pytest.skip("Failed to build llmspell")

    # Verify the binary exists
    if not binary_path.exists():
        pytest.skip(f"llmspell binary not found at {binary_path}")

    return binary_path


@pytest.fixture(scope="session")
def llmspell_daemon(build_llmspell):
    """
    Start llmspell daemon for the test session with proper cleanup.

    This fixture:
    1. Creates a temporary directory for test artifacts
    2. Starts llmspell in daemon mode with a connection file
    3. Waits for the kernel to be ready
    4. Yields kernel information for tests
    5. Ensures proper cleanup on teardown
    """
    with tempfile.TemporaryDirectory(prefix="llmspell_test_") as tmpdir:
        tmpdir = Path(tmpdir)
        connection_file = tmpdir / "kernel.json"
        log_file = tmpdir / "kernel.log"
        pid_file = tmpdir / "kernel.pid"

        # Start daemon with connection file
        # Using port 0 lets the OS assign an available port
        cmd = [
            str(build_llmspell), "kernel", "start",
            "--daemon",
            "--port", "0",  # Let OS assign port to avoid conflicts
            "--connection-file", str(connection_file),
            "--log-file", str(log_file),
            "--idle-timeout", "300"  # 5 min timeout for tests
        ]

        logger.info(f"Starting llmspell daemon: {' '.join(cmd)}")
        logger.info(f"Connection file: {connection_file}")
        logger.info(f"Log file: {log_file}")

        # Start the daemon process
        proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True
        )
        _spawned_processes.append(proc)

        # Wait for connection file to be written (max 30s)
        start_time = time.time()
        while time.time() - start_time < 30:
            if connection_file.exists():
                # Verify it's valid JSON
                try:
                    with open(connection_file) as f:
                        conn_info = json.load(f)
                    logger.info(f"Kernel started successfully on ports: {conn_info}")
                    break
                except (json.JSONDecodeError, KeyError) as e:
                    # File might be partially written, wait a bit more
                    time.sleep(0.1)
                    continue

            # Check if process died
            # In daemon mode, the parent process exits with code 0 after forking
            if proc.poll() is not None:
                exit_code = proc.returncode
                stdout, stderr = proc.communicate()

                # Exit code 0 is expected for daemon mode - parent exits after fork
                if exit_code == 0:
                    logger.debug("Parent process exited normally (daemon mode)")
                    # Give daemon child a moment to create connection file
                    time.sleep(0.5)
                    continue
                else:
                    # Non-zero exit code indicates actual failure
                    logger.error(f"Kernel process died. stdout: {stdout}")
                    logger.error(f"stderr: {stderr}")
                    if log_file.exists():
                        logger.error(f"Log file contents:\n{log_file.read_text()}")
                    raise RuntimeError(f"Kernel failed to start. Exit code: {exit_code}")

            time.sleep(0.1)
        else:
            # Timeout reached
            proc.terminate()
            proc.wait()
            if log_file.exists():
                logger.error(f"Log file contents:\n{log_file.read_text()}")
            raise RuntimeError("Kernel failed to start within 30 seconds")

        # Get actual daemon PID from PID file (since parent process exited)
        kernel_pid = None
        if pid_file.exists():
            try:
                kernel_pid = int(pid_file.read_text().strip())
                logger.info(f"Kernel daemon PID: {kernel_pid}")
            except (ValueError, FileNotFoundError):
                logger.warning("Could not read PID from PID file")

        yield {
            "process": proc,
            "connection_file": connection_file,
            "log_file": log_file,
            "tmpdir": tmpdir,
            "connection_info": conn_info,
            "kernel_pid": kernel_pid,
            "pid_file": pid_file
        }

        # Cleanup: terminate daemon gracefully
        logger.info("Shutting down kernel daemon...")

        # For daemon mode, use the PID from PID file to kill the actual daemon process
        if kernel_pid:
            try:
                os.kill(kernel_pid, signal.SIGTERM)
                # Wait a bit for graceful shutdown
                for _ in range(10):
                    try:
                        os.kill(kernel_pid, 0)  # Check if process still exists
                        time.sleep(1)
                    except ProcessLookupError:
                        logger.info("Kernel daemon terminated gracefully")
                        break
                else:
                    # Force kill if still running
                    logger.warning("Kernel didn't terminate gracefully, forcing kill")
                    os.kill(kernel_pid, signal.SIGKILL)
            except ProcessLookupError:
                logger.info("Kernel daemon already terminated")
            except Exception as e:
                logger.error(f"Error terminating kernel: {e}")

        # Clean up PID file
        if pid_file.exists():
            pid_file.unlink()

        # Log final state if there were issues
        if log_file.exists():
            log_contents = log_file.read_text()
            if "ERROR" in log_contents or "PANIC" in log_contents:
                logger.error(f"Errors found in kernel log:\n{log_contents}")


@pytest.fixture
def kernel_client(llmspell_daemon):
    """
    Create a Jupyter kernel client for each test.

    This fixture:
    1. Creates a BlockingKernelClient using the connection file
    2. Starts communication channels
    3. Waits for the kernel to be ready
    4. Yields the client for test use
    5. Properly closes channels on cleanup
    """
    connection_file = llmspell_daemon["connection_file"]

    logger.info(f"Creating kernel client with connection file: {connection_file}")

    # Create client with connection file
    client = BlockingKernelClient(connection_file=str(connection_file))

    # Start channels
    client.start_channels()

    # Wait for kernel to be ready
    try:
        client.wait_for_ready(timeout=10)
        logger.info("Kernel client ready")
    except Exception as e:
        client.stop_channels()
        log_file = llmspell_daemon["log_file"]
        if log_file.exists():
            logger.error(f"Kernel log:\n{log_file.read_text()}")
        raise RuntimeError(f"Kernel not ready: {e}")

    yield client

    # Cleanup client connections
    logger.info("Closing kernel client channels")
    client.stop_channels()


@pytest.fixture
def test_script_dir(tmp_path):
    """
    Create a temporary directory for test scripts.

    Returns a Path object to a temporary directory that will be
    cleaned up after the test.
    """
    return tmp_path


@pytest.fixture(autouse=True)
def log_test_info(request):
    """Automatically log test start and end for debugging."""
    logger.info(f"\n{'='*60}")
    logger.info(f"Starting test: {request.node.name}")
    logger.info(f"{'='*60}")
    yield
    logger.info(f"Finished test: {request.node.name}\n")


@pytest.fixture
def timeout_handler():
    """
    Provide a context manager for handling test timeouts.

    Usage:
        with timeout_handler(5):
            # code that should complete within 5 seconds
    """
    import signal
    from contextlib import contextmanager

    @contextmanager
    def timeout(seconds):
        def timeout_handler(signum, frame):
            raise TimeoutError(f"Operation timed out after {seconds} seconds")

        # Set up the timeout
        old_handler = signal.signal(signal.SIGALRM, timeout_handler)
        signal.alarm(seconds)

        try:
            yield
        finally:
            # Restore the old handler and cancel the alarm
            signal.alarm(0)
            signal.signal(signal.SIGALRM, old_handler)

    return timeout
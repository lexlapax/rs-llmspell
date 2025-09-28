#!/usr/bin/env python3
"""
llmspell-fleet manager - Python implementation with better process management
"""

import json
import os
import signal
import subprocess
import sys
import time
import uuid
from datetime import datetime, timezone
from pathlib import Path
from typing import Dict, List, Optional, Any

import psutil

class FleetManager:
    """Manage multiple LLMSpell kernel processes"""

    def __init__(self, fleet_dir: Path = None):
        self.fleet_dir = fleet_dir or Path.home() / ".llmspell" / "fleet"
        self.fleet_dir.mkdir(parents=True, exist_ok=True)

        self.registry_file = self.fleet_dir / "registry.json"
        self.log_dir = self.fleet_dir / "logs"
        self.log_dir.mkdir(exist_ok=True)

        self.load_registry()

    def load_registry(self):
        """Load or initialize the kernel registry"""
        if self.registry_file.exists():
            with open(self.registry_file) as f:
                self.registry = json.load(f)
        else:
            self.registry = {
                "kernels": [],
                "next_port": 9555,
                "total_spawned": 0
            }
            self.save_registry()

    def save_registry(self):
        """Save registry to disk atomically"""
        temp_file = self.registry_file.with_suffix('.tmp')
        with open(temp_file, 'w') as f:
            json.dump(self.registry, f, indent=2, default=str)
        temp_file.replace(self.registry_file)

    def find_free_port(self, start_port: int = None) -> int:
        """Find next available port"""
        import socket
        port = start_port or self.registry.get("next_port", 9555)

        while port < 10000:
            try:
                with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                    s.bind(('', port))
                    return port
            except OSError:
                port += 1

        raise RuntimeError("No free ports available")

    def spawn_kernel(
        self,
        config_file: str = "default.toml",
        language: str = "lua",
        memory_limit: str = None,
        cpu_limit: float = None,
        env_vars: Dict[str, str] = None
    ) -> Dict[str, Any]:
        """Spawn a new kernel process"""

        kernel_id = f"kernel-{uuid.uuid4().hex[:8]}"
        port = self.find_free_port()

        # File paths
        pid_file = self.fleet_dir / f"{kernel_id}.pid"
        log_file = self.log_dir / f"{kernel_id}.log"
        connection_file = self.fleet_dir / f"{kernel_id}.json"

        # Find llmspell binary
        llmspell_bin = None
        if subprocess.run(["which", "llmspell"], capture_output=True).returncode == 0:
            llmspell_bin = "llmspell"
        elif Path("../../target/release/llmspell").is_file():
            llmspell_bin = str(Path("../../target/release/llmspell").absolute())
        elif Path("../../target/debug/llmspell").is_file():
            llmspell_bin = str(Path("../../target/debug/llmspell").absolute())
        else:
            print("ERROR: llmspell binary not found")
            return None

        # Build command with proper daemon parameters
        cmd = [
            llmspell_bin, "kernel", "start",
            "--daemon",
            "--port", str(port),
            "--connection-file", str(connection_file),
            "--log-file", str(log_file),
            "--pid-file", str(pid_file)
        ]

        # Only add config if it's not default and exists
        if config_file != "default.toml" and Path(f"configs/{config_file}").exists():
            cmd.extend(["--config", f"configs/{config_file}"])

        # Set up environment
        env = os.environ.copy()
        if env_vars:
            env.update(env_vars)

        print(f"Spawning kernel {kernel_id} on port {port}...")

        # Start daemon process (it will handle its own backgrounding)
        result = subprocess.run(cmd, env=env, capture_output=True, text=True)

        if result.returncode != 0:
            print(f"ERROR: Failed to start kernel: {result.stderr}")
            return None

        # Wait for PID file to be created by daemon
        for _ in range(10):
            if pid_file.exists():
                pid = int(pid_file.read_text().strip())
                break
            time.sleep(0.1)
        else:
            print(f"ERROR: PID file not created at {pid_file}")
            return None

        # Wait for kernel to start
        if not self.wait_for_kernel(port, timeout=15):
            print(f"ERROR: Kernel failed to start on port {port}")
            try:
                process.kill()
            except:
                pass
            return None

        # Apply resource limits if requested
        if memory_limit or cpu_limit:
            self.apply_resource_limits(pid, memory_limit, cpu_limit)

        # Update registry
        kernel_info = {
            "id": kernel_id,
            "pid": pid,
            "port": port,
            "language": language,
            "config": config_file,
            "connection_file": str(connection_file),
            "log_file": str(log_file),
            "started_at": datetime.now(timezone.utc).isoformat(),
            "status": "running",
            "memory_limit": memory_limit,
            "cpu_limit": cpu_limit,
            "clients": []
        }

        self.registry["kernels"].append(kernel_info)
        self.registry["next_port"] = port + 1
        self.registry["total_spawned"] = self.registry.get("total_spawned", 0) + 1
        self.save_registry()

        print(f"✓ Kernel {kernel_id} started")
        print(f"  Port: {port}")
        print(f"  PID: {pid}")
        print(f"  Config: {config_file}")
        print(f"  Connection: {connection_file}")
        print(f"  Logs: {log_file}")

        return kernel_info

    def wait_for_kernel(self, port: int, timeout: int = 15) -> bool:
        """Wait for kernel to start listening on port"""
        import socket
        start_time = time.time()

        while time.time() - start_time < timeout:
            try:
                with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                    s.settimeout(1)
                    s.connect(('localhost', port))
                    return True
            except (socket.error, socket.timeout):
                time.sleep(0.5)

        return False

    def apply_resource_limits(self, pid: int, memory_limit: str = None, cpu_limit: float = None):
        """Apply resource limits to process (Linux only)"""
        try:
            process = psutil.Process(pid)

            if memory_limit:
                # Parse memory limit (e.g., "512M", "1G")
                import re
                match = re.match(r'(\d+)([MGK]?)', memory_limit.upper())
                if match:
                    value, unit = match.groups()
                    value = int(value)
                    if unit == 'G':
                        value *= 1024 * 1024 * 1024
                    elif unit == 'M':
                        value *= 1024 * 1024
                    elif unit == 'K':
                        value *= 1024

                    # Use cgroups on Linux
                    if sys.platform == 'linux':
                        cgroup_path = f"/sys/fs/cgroup/memory/llmspell/{pid}"
                        os.makedirs(cgroup_path, exist_ok=True)
                        with open(f"{cgroup_path}/memory.limit_in_bytes", 'w') as f:
                            f.write(str(value))
                        with open(f"{cgroup_path}/tasks", 'w') as f:
                            f.write(str(pid))

            if cpu_limit:
                # Set CPU affinity or nice value
                process.nice(10)  # Lower priority

                # Use cgroups on Linux for CPU quota
                if sys.platform == 'linux':
                    cgroup_path = f"/sys/fs/cgroup/cpu/llmspell/{pid}"
                    os.makedirs(cgroup_path, exist_ok=True)
                    # cpu_limit is a fraction (0.5 = 50%)
                    quota = int(cpu_limit * 100000)  # 100ms period
                    with open(f"{cgroup_path}/cpu.cfs_quota_us", 'w') as f:
                        f.write(str(quota))
                    with open(f"{cgroup_path}/tasks", 'w') as f:
                        f.write(str(pid))

        except Exception as e:
            print(f"Warning: Could not apply resource limits: {e}")

    def list_kernels(self, verbose: bool = False) -> List[Dict]:
        """List all kernels with their status"""
        self.cleanup_dead_kernels()

        print("Active Kernels:")
        print("───────────────")

        if not self.registry["kernels"]:
            print("No kernels running")
            return []

        active_kernels = []
        for kernel in self.registry["kernels"]:
            pid = kernel["pid"]

            # Check if process is running
            try:
                process = psutil.Process(pid)
                status = "✓ running"
                memory_mb = process.memory_info().rss / 1024 / 1024
                cpu_percent = process.cpu_percent(interval=0.1)

                if verbose:
                    connections = len(process.net_connections())
                    threads = process.num_threads()
            except psutil.NoSuchProcess:
                status = "✗ dead"
                memory_mb = 0
                cpu_percent = 0
                connections = 0
                threads = 0

            print(f"{kernel['id']:12}  PID:{pid:7}  Port:{kernel['port']:5}  "
                  f"Lang:{kernel['language']:10}  Mem:{memory_mb:6.1f}MB  "
                  f"CPU:{cpu_percent:5.1f}%  {status}")

            if verbose and status == "✓ running":
                print(f"              Connections:{connections:3}  Threads:{threads:3}  "
                      f"Started: {kernel['started_at']}")

            if status == "✓ running":
                active_kernels.append(kernel)

        return active_kernels

    def stop_kernel(self, kernel_id: str, force: bool = False) -> bool:
        """Stop a kernel gracefully"""
        # Support stopping by port
        if kernel_id.isdigit():
            port = int(kernel_id)
            kernel = next((k for k in self.registry["kernels"]
                          if k["port"] == port), None)
            if kernel:
                kernel_id = kernel["id"]

        kernel = next((k for k in self.registry["kernels"]
                      if k["id"] == kernel_id), None)

        if not kernel:
            print(f"Kernel {kernel_id} not found")
            return False

        pid = kernel["pid"]
        print(f"Stopping kernel {kernel_id} (PID: {pid})...")

        try:
            process = psutil.Process(pid)

            if not force:
                # Graceful shutdown
                process.terminate()
                try:
                    process.wait(timeout=5)
                except psutil.TimeoutExpired:
                    print("  Graceful shutdown timeout, force killing...")
                    process.kill()
            else:
                process.kill()

            # Wait for process to die
            process.wait(timeout=2)

        except psutil.NoSuchProcess:
            pass  # Already dead
        except Exception as e:
            print(f"  Error stopping process: {e}")
            return False

        # Remove from registry
        self.registry["kernels"] = [k for k in self.registry["kernels"]
                                    if k["id"] != kernel_id]
        self.save_registry()

        # Clean up files
        for suffix in ['.pid', '.json']:
            file_path = self.fleet_dir / f"{kernel_id}{suffix}"
            file_path.unlink(missing_ok=True)

        print(f"✓ Kernel {kernel_id} stopped")
        return True

    def stop_all(self, force: bool = False):
        """Stop all kernels"""
        kernel_ids = [k["id"] for k in self.registry["kernels"]]

        if not kernel_ids:
            print("No kernels to stop")
            return

        print(f"Stopping {len(kernel_ids)} kernels...")
        for kernel_id in kernel_ids:
            self.stop_kernel(kernel_id, force)

    def cleanup_dead_kernels(self):
        """Remove dead kernels from registry"""
        alive_kernels = []

        for kernel in self.registry["kernels"]:
            try:
                psutil.Process(kernel["pid"])
                alive_kernels.append(kernel)
            except psutil.NoSuchProcess:
                print(f"  Removing dead kernel {kernel['id']} (PID: {kernel['pid']})")

        if len(alive_kernels) != len(self.registry["kernels"]):
            self.registry["kernels"] = alive_kernels
            self.save_registry()

    def find_or_create_kernel(
        self,
        requirements: Dict[str, Any]
    ) -> Optional[Dict[str, Any]]:
        """Find a matching kernel or create a new one"""

        # Clean up dead kernels first
        self.cleanup_dead_kernels()

        language = requirements.get("language", "lua")
        config = requirements.get("config", "default.toml")

        # Find matching kernel
        for kernel in self.registry["kernels"]:
            if (kernel["language"] == language and
                kernel["config"] == config and
                kernel.get("status") == "running"):

                # Verify it's actually running
                try:
                    psutil.Process(kernel["pid"])
                    print(f"Found matching kernel: {kernel['id']}")
                    return kernel
                except psutil.NoSuchProcess:
                    pass

        # No matching kernel, spawn new one
        print("No matching kernel found, spawning new one...")
        return self.spawn_kernel(
            config_file=config,
            language=language,
            memory_limit=requirements.get("memory_limit"),
            cpu_limit=requirements.get("cpu_limit"),
            env_vars=requirements.get("env_vars")
        )

    def get_metrics(self) -> Dict[str, Any]:
        """Collect metrics for all kernels"""
        metrics = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "total_kernels": len(self.registry["kernels"]),
            "total_spawned": self.registry.get("total_spawned", 0),
            "kernels": []
        }

        for kernel in self.registry["kernels"]:
            try:
                process = psutil.Process(kernel["pid"])
                kernel_metrics = {
                    "id": kernel["id"],
                    "port": kernel["port"],
                    "language": kernel["language"],
                    "memory_mb": process.memory_info().rss / 1024 / 1024,
                    "cpu_percent": process.cpu_percent(interval=0.1),
                    "connections": len(process.connections()),
                    "threads": process.num_threads(),
                    "uptime_seconds": time.time() - process.create_time()
                }
                metrics["kernels"].append(kernel_metrics)
            except psutil.NoSuchProcess:
                pass

        return metrics


def main():
    """Command-line interface"""
    import argparse

    parser = argparse.ArgumentParser(description="LLMSpell Fleet Manager")
    subparsers = parser.add_subparsers(dest='command', help='Commands')

    # Spawn command
    spawn_parser = subparsers.add_parser('spawn', help='Spawn a new kernel')
    spawn_parser.add_argument('--config', default='default.toml', help='Config file')
    spawn_parser.add_argument('--language', default='lua', help='Language (lua/javascript/python)')
    spawn_parser.add_argument('--memory', help='Memory limit (e.g., 512M, 1G)')
    spawn_parser.add_argument('--cpu', type=float, help='CPU limit (0.5 = 50%)')

    # List command
    list_parser = subparsers.add_parser('list', help='List kernels')
    list_parser.add_argument('-v', '--verbose', action='store_true', help='Verbose output')

    # Stop command
    stop_parser = subparsers.add_parser('stop', help='Stop a kernel')
    stop_parser.add_argument('kernel', help='Kernel ID or port')
    stop_parser.add_argument('-f', '--force', action='store_true', help='Force kill')

    # Stop all command
    stopall_parser = subparsers.add_parser('stop-all', help='Stop all kernels')
    stopall_parser.add_argument('-f', '--force', action='store_true', help='Force kill')

    # Cleanup command
    subparsers.add_parser('cleanup', help='Clean up dead kernels')

    # Find command
    find_parser = subparsers.add_parser('find', help='Find or create matching kernel')
    find_parser.add_argument('--language', default='lua', help='Language requirement')
    find_parser.add_argument('--config', default='default.toml', help='Config requirement')

    # Metrics command
    subparsers.add_parser('metrics', help='Show kernel metrics')

    args = parser.parse_args()

    fleet = FleetManager()

    if args.command == 'spawn':
        fleet.spawn_kernel(
            config_file=args.config,
            language=args.language,
            memory_limit=args.memory,
            cpu_limit=args.cpu
        )
    elif args.command == 'list':
        fleet.list_kernels(verbose=args.verbose)
    elif args.command == 'stop':
        fleet.stop_kernel(args.kernel, force=args.force)
    elif args.command == 'stop-all':
        fleet.stop_all(force=args.force)
    elif args.command == 'cleanup':
        fleet.cleanup_dead_kernels()
        print("✓ Cleanup complete")
    elif args.command == 'find':
        kernel = fleet.find_or_create_kernel({
            "language": args.language,
            "config": args.config
        })
        if kernel:
            print(f"Connection file: {kernel['connection_file']}")
    elif args.command == 'metrics':
        metrics = fleet.get_metrics()
        print(json.dumps(metrics, indent=2))
    else:
        parser.print_help()


if __name__ == '__main__':
    main()
#!/usr/bin/env python3
import psutil
import json
import time
import sys

def monitor_kernel_resources():
    """Monitor resource usage of all kernel processes"""
    print("Monitoring kernel resources (5 second intervals)...")
    print("=" * 60)

    for _ in range(3):  # Monitor for 15 seconds
        kernels = []
        for proc in psutil.process_iter(['pid', 'name', 'cmdline']):
            try:
                if 'llmspell' in ' '.join(proc.info['cmdline'] or []) and 'kernel' in ' '.join(proc.info['cmdline'] or []):
                    mem_info = proc.memory_info()
                    cpu_percent = proc.cpu_percent(interval=0.1)
                    io_counters = proc.io_counters() if hasattr(proc, 'io_counters') else None

                    kernel_info = {
                        'pid': proc.pid,
                        'memory_mb': round(mem_info.rss / 1024 / 1024, 2),
                        'memory_percent': round(proc.memory_percent(), 2),
                        'cpu_percent': round(cpu_percent, 2),
                        'num_threads': proc.num_threads(),
                        'nice': proc.nice(),
                    }

                    if io_counters:
                        kernel_info['io_read_mb'] = round(io_counters.read_bytes / 1024 / 1024, 2)
                        kernel_info['io_write_mb'] = round(io_counters.write_bytes / 1024 / 1024, 2)

                    kernels.append(kernel_info)
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                continue

        if kernels:
            print(f"\nTimestamp: {time.strftime('%Y-%m-%d %H:%M:%S')}")
            for k in kernels:
                print(f"  PID {k['pid']}: "
                      f"Mem: {k['memory_mb']}MB ({k['memory_percent']}%), "
                      f"CPU: {k['cpu_percent']}%, "
                      f"Nice: {k['nice']}, "
                      f"Threads: {k['num_threads']}")
        else:
            print("No kernel processes found")

        time.sleep(5)

    print("=" * 60)

if __name__ == "__main__":
    monitor_kernel_resources()

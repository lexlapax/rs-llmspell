#!/bin/bash
# Fleet Resource Management Examples
# Shows how to apply resource limits at OS and container levels

set -e

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "=== Fleet Resource Management Examples ==="
echo ""

cd "$(dirname "$0")/.."

# Clean start
./llmspell-fleet stop-all 2>/dev/null || true

echo -e "${GREEN}Example 1: Memory Limits with ulimit${NC}"
echo "======================================"
echo ""

# Check current limits
echo "Current memory limits:"
ulimit -a | grep -E "memory|data" || echo "Memory limits not shown"
echo ""

# Example with memory limit (macOS doesn't support ulimit -m directly)
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "Linux: Setting memory limit to 512MB"
    echo "(ulimit -m 524288 && ./llmspell-fleet spawn)"

    # Actually test it
    (ulimit -v 524288 2>/dev/null && ./llmspell-fleet spawn) || \
        echo "Note: Memory limits may require root or cgroup configuration"
else
    echo "macOS: Memory limits via ulimit not directly supported"
    echo "Use Docker or Activity Monitor for resource management"
fi
echo ""

echo -e "${GREEN}Example 2: CPU Priority with nice${NC}"
echo "=================================="
echo ""

echo "Spawning low-priority kernel (nice value 10):"
nice -n 10 ./llmspell-fleet spawn
NICE_KERNEL=$(./llmspell-fleet list | grep kernel- | tail -1 | awk '{print $1}')
NICE_PID=$(./llmspell-fleet list | grep "$NICE_KERNEL" | awk '{print $2}' | sed 's/PID://')

echo ""
echo "Checking nice value:"
ps -o pid,nice,comm -p $NICE_PID 2>/dev/null || echo "Process info not available"
echo -e "${GREEN}✓ Low-priority kernel spawned: $NICE_KERNEL${NC}"
echo ""

echo -e "${GREEN}Example 3: Process Resource Monitoring${NC}"
echo "======================================="
echo ""

# Create monitoring script
cat > monitor_resources.py << 'EOF'
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
EOF

echo "Running resource monitor..."
python3 monitor_resources.py

echo ""
echo -e "${GREEN}Example 4: Resource Limits via cgroups (Linux)${NC}"
echo "=============================================="
echo ""

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    cat << 'EOF'
# Create cgroup for llmspell kernels
sudo cgcreate -g memory,cpu:llmspell

# Set memory limit to 512MB
echo 536870912 | sudo tee /sys/fs/cgroup/memory/llmspell/memory.limit_in_bytes

# Set CPU quota (50% of one CPU)
echo 50000 | sudo tee /sys/fs/cgroup/cpu/llmspell/cpu.cfs_quota_us
echo 100000 | sudo tee /sys/fs/cgroup/cpu/llmspell/cpu.cfs_period_us

# Run kernel in cgroup
sudo cgexec -g memory,cpu:llmspell ./llmspell-fleet spawn

# Monitor cgroup resources
cat /sys/fs/cgroup/memory/llmspell/memory.usage_in_bytes
cat /sys/fs/cgroup/cpu/llmspell/cpuacct.usage
EOF
else
    echo "cgroups are Linux-specific. On macOS, use Docker for similar functionality."
fi
echo ""

echo -e "${GREEN}Example 5: Docker Resource Constraints${NC}"
echo "======================================="
echo ""

# Show docker-compose resource configuration
echo "Docker Compose with resource limits (from docker-compose.yml):"
grep -A3 -B1 "mem_limit\|cpus:" docker-compose.yml | head -20
echo ""

echo "To run with Docker resource limits:"
echo "  docker-compose up kernel-lua-openai  # 512MB memory, 0.5 CPUs"
echo ""

echo -e "${GREEN}Example 6: Resource Cleanup Script${NC}"
echo "==================================="
echo ""

cat > cleanup_resources.sh << 'EOF'
#!/bin/bash
# Clean up resources and zombie processes

echo "Cleaning up kernel resources..."

# Stop all kernels gracefully
./llmspell-fleet stop-all

# Kill any remaining llmspell processes
pkill -f "llmspell kernel" 2>/dev/null || true

# Clean up PID files
rm -f ~/.llmspell/fleet/*.pid

# Clean up stale connection files
find ~/.llmspell/fleet -name "*.json" -mtime +1 -delete 2>/dev/null || true

# Report on disk usage
echo "Fleet disk usage:"
du -sh ~/.llmspell/fleet/ 2>/dev/null || echo "No fleet directory"

echo "✓ Resource cleanup complete"
EOF

chmod +x cleanup_resources.sh
echo "Created cleanup_resources.sh for resource management"
echo ""

echo -e "${GREEN}Example 7: Load Testing with Resource Monitoring${NC}"
echo "================================================"
echo ""

cat > load_test.sh << 'EOF'
#!/bin/bash
# Spawn multiple kernels and monitor resource usage

echo "Load test: Spawning 5 kernels..."

for i in {1..5}; do
    echo "Spawning kernel $i..."
    ./llmspell-fleet spawn &
    sleep 2
done

wait

echo ""
echo "All kernels spawned. Resource usage:"
python3 fleet_manager.py metrics | jq -r '
    "Total Kernels: \(.total_kernels)",
    "Total Memory: \([.kernels[].memory_mb] | add | round)MB",
    "Average Memory per Kernel: \(([.kernels[].memory_mb] | add / length) | round)MB",
    "Total CPU: \([.kernels[].cpu_percent] | add | round)%",
    "Average CPU per Kernel: \(([.kernels[].cpu_percent] | add / length) | round)%"
'

echo ""
echo "Stopping all test kernels..."
./llmspell-fleet stop-all
EOF

chmod +x load_test.sh
echo "Created load_test.sh for stress testing"
echo ""

echo -e "${YELLOW}Resource Management Best Practices:${NC}"
echo "===================================="
echo "1. Memory Management:"
echo "   - Monitor baseline usage (~45MB per kernel)"
echo "   - Set limits 2x baseline for safety"
echo "   - Use swap monitoring on Linux"
echo ""
echo "2. CPU Management:"
echo "   - Use nice for priority control"
echo "   - Implement CPU quotas via cgroups"
echo "   - Monitor for CPU throttling"
echo ""
echo "3. I/O Management:"
echo "   - Monitor disk usage in logs directory"
echo "   - Implement log rotation"
echo "   - Set connection file cleanup policies"
echo ""
echo "4. Process Management:"
echo "   - Regular cleanup of dead kernels"
echo "   - PID file verification"
echo "   - Graceful shutdown procedures"
echo ""

echo -e "${GREEN}Current Resource Usage:${NC}"
echo "========================"
./llmspell-fleet list
echo ""

# Show total resource usage
echo "Fleet-wide resource summary:"
ps aux | grep "llmspell kernel" | grep -v grep | awk '
    {mem+=$6; cpu+=$3; count++}
    END {
        if (count > 0) {
            print "  Total Memory: " int(mem/1024) "MB"
            print "  Total CPU: " cpu "%"
            print "  Kernel Count: " count
            print "  Avg Memory/Kernel: " int(mem/1024/count) "MB"
        } else {
            print "  No kernels running"
        }
    }
'

echo ""
echo -e "${GREEN}✓ Resource management examples complete!${NC}"
echo ""
echo "Cleanup kernel:"
./llmspell-fleet stop $NICE_KERNEL 2>/dev/null || true
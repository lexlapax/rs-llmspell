#!/usr/bin/env python3
"""
Log Aggregator - Centralized log collection and monitoring for LLMSpell kernel fleet
"""

import json
import time
import re
import os
import argparse
from datetime import datetime, timedelta
from pathlib import Path
from typing import Dict, List, Any, Optional
from collections import defaultdict, deque

class LogAggregator:
    """Aggregate and analyze logs from multiple kernel processes"""

    def __init__(self, fleet_dir: Path = None):
        self.fleet_dir = fleet_dir or Path.home() / ".llmspell" / "fleet"
        self.log_dir = self.fleet_dir / "logs"
        self.registry_file = self.fleet_dir / "registry.json"

        # Log patterns for error detection
        self.error_patterns = [
            (re.compile(r'ERROR|FATAL', re.I), 'ERROR'),
            (re.compile(r'WARN|WARNING', re.I), 'WARNING'),
            (re.compile(r'panic|crash|abort', re.I), 'CRITICAL'),
            (re.compile(r'timeout|timed out', re.I), 'TIMEOUT'),
            (re.compile(r'connection refused|connection failed', re.I), 'CONNECTION'),
            (re.compile(r'out of memory|OOM', re.I), 'MEMORY'),
            (re.compile(r'permission denied|access denied', re.I), 'PERMISSION'),
        ]

        # Performance patterns
        self.perf_patterns = [
            (re.compile(r'took (\d+)ms'), 'execution_time'),
            (re.compile(r'latency: (\d+)ms'), 'latency'),
            (re.compile(r'memory: (\d+)MB'), 'memory_usage'),
            (re.compile(r'cpu: (\d+)%'), 'cpu_usage'),
        ]

        # Retention policy (hours)
        self.retention_hours = 24

        # Alert state
        self.recent_errors = deque(maxlen=100)
        self.error_counts = defaultdict(int)
        self.alert_thresholds = {
            'ERROR': 10,       # Alert after 10 errors
            'CRITICAL': 1,     # Alert immediately on critical
            'WARNING': 20,     # Alert after 20 warnings
            'TIMEOUT': 5,      # Alert after 5 timeouts
        }

    def load_registry(self) -> Dict:
        """Load kernel registry"""
        if not self.registry_file.exists():
            return {"kernels": []}

        with open(self.registry_file) as f:
            return json.load(f)

    def get_log_files(self) -> List[Path]:
        """Get all kernel log files"""
        if not self.log_dir.exists():
            return []

        return list(self.log_dir.glob("kernel-*.log"))

    def tail_file(self, file_path: Path, lines: int = 100) -> List[str]:
        """Tail last N lines from a file"""
        try:
            with open(file_path, 'r') as f:
                # Efficient tail implementation
                f.seek(0, 2)  # Go to end
                file_size = f.tell()

                # Read from end
                block_size = min(8192, file_size)
                blocks = []
                read_size = 0

                while read_size < file_size and len(''.join(blocks).splitlines()) < lines:
                    read_size = min(read_size + block_size, file_size)
                    f.seek(file_size - read_size)
                    blocks.insert(0, f.read(min(block_size, read_size)))

                all_lines = ''.join(blocks).splitlines()
                return all_lines[-lines:] if len(all_lines) > lines else all_lines

        except (IOError, OSError):
            return []

    def parse_log_line(self, line: str) -> Dict[str, Any]:
        """Parse a log line for relevant information"""
        result = {
            'raw': line,
            'timestamp': None,
            'level': 'INFO',
            'message': line,
            'metrics': {}
        }

        # Try to extract timestamp
        timestamp_match = re.match(r'^(\d{4}-\d{2}-\d{2}[T\s]\d{2}:\d{2}:\d{2})', line)
        if timestamp_match:
            result['timestamp'] = timestamp_match.group(1)
            result['message'] = line[len(timestamp_match.group(1)):].strip()

        # Check for error patterns
        for pattern, level in self.error_patterns:
            if pattern.search(line):
                result['level'] = level
                break

        # Extract performance metrics
        for pattern, metric_name in self.perf_patterns:
            match = pattern.search(line)
            if match:
                result['metrics'][metric_name] = match.group(1)

        return result

    def aggregate_logs(self, tail_lines: int = 100) -> Dict[str, Any]:
        """Aggregate logs from all kernels"""
        registry = self.load_registry()
        log_files = self.get_log_files()

        aggregated = {
            'timestamp': datetime.now().isoformat(),
            'kernels': {},
            'summary': {
                'total_kernels': len(registry.get('kernels', [])),
                'total_log_files': len(log_files),
                'error_count': 0,
                'warning_count': 0,
                'recent_errors': []
            }
        }

        # Process each kernel's logs
        for kernel in registry.get('kernels', []):
            kernel_id = kernel['id']
            log_file = self.log_dir / f"{kernel_id}.log"

            if log_file.exists():
                lines = self.tail_file(log_file, tail_lines)
                parsed_lines = [self.parse_log_line(line) for line in lines]

                # Count errors/warnings
                errors = [l for l in parsed_lines if l['level'] in ['ERROR', 'CRITICAL']]
                warnings = [l for l in parsed_lines if l['level'] == 'WARNING']

                aggregated['kernels'][kernel_id] = {
                    'port': kernel['port'],
                    'log_file': str(log_file),
                    'total_lines': len(lines),
                    'errors': len(errors),
                    'warnings': len(warnings),
                    'recent_logs': parsed_lines[-10:],  # Last 10 lines
                    'error_logs': errors[-5:]  # Last 5 errors
                }

                # Update summary
                aggregated['summary']['error_count'] += len(errors)
                aggregated['summary']['warning_count'] += len(warnings)

                # Track recent errors globally
                for error in errors:
                    self.recent_errors.append({
                        'kernel_id': kernel_id,
                        'timestamp': error.get('timestamp', 'unknown'),
                        'message': error['message']
                    })

        # Add recent errors to summary
        aggregated['summary']['recent_errors'] = list(self.recent_errors)[-10:]

        return aggregated

    def search_logs(self, pattern: str, kernel_id: Optional[str] = None, context_lines: int = 2) -> List[Dict]:
        """Search logs for specific pattern"""
        results = []
        regex = re.compile(pattern, re.I)

        if kernel_id:
            # Search specific kernel
            log_file = self.log_dir / f"kernel-{kernel_id}.log"
            if log_file.exists():
                results.extend(self._search_file(log_file, regex, kernel_id, context_lines))
        else:
            # Search all logs
            for log_file in self.get_log_files():
                kernel_id = log_file.stem
                results.extend(self._search_file(log_file, regex, kernel_id, context_lines))

        return results

    def _search_file(self, file_path: Path, regex, kernel_id: str, context_lines: int) -> List[Dict]:
        """Search a single log file"""
        results = []

        try:
            with open(file_path, 'r') as f:
                lines = f.readlines()

            for i, line in enumerate(lines):
                if regex.search(line):
                    # Get context
                    start = max(0, i - context_lines)
                    end = min(len(lines), i + context_lines + 1)
                    context = lines[start:end]

                    results.append({
                        'kernel_id': kernel_id,
                        'file': str(file_path),
                        'line_number': i + 1,
                        'match': line.strip(),
                        'context': [l.strip() for l in context]
                    })

        except (IOError, OSError):
            pass

        return results

    def monitor_errors(self, callback=None) -> Dict[str, int]:
        """Monitor error rates and trigger alerts"""
        aggregated = self.aggregate_logs(tail_lines=500)

        # Count errors by type
        current_errors = defaultdict(int)
        for kernel_data in aggregated['kernels'].values():
            for log in kernel_data.get('recent_logs', []):
                if log['level'] in self.error_counts:
                    current_errors[log['level']] += 1

        # Check thresholds
        alerts = []
        for error_type, count in current_errors.items():
            if count >= self.alert_thresholds.get(error_type, float('inf')):
                alert_msg = f"ALERT: {error_type} threshold exceeded ({count} occurrences)"
                alerts.append(alert_msg)

                if callback:
                    callback(error_type, count)

        # Update error counts
        self.error_counts.update(current_errors)

        return {
            'errors': dict(current_errors),
            'alerts': alerts,
            'total_errors': aggregated['summary']['error_count'],
            'total_warnings': aggregated['summary']['warning_count']
        }

    def rotate_logs(self):
        """Rotate old log files based on retention policy"""
        cutoff_time = datetime.now() - timedelta(hours=self.retention_hours)

        rotated = []
        for log_file in self.get_log_files():
            # Check file age
            if log_file.stat().st_mtime < cutoff_time.timestamp():
                # Archive or delete old log
                archive_path = log_file.with_suffix('.log.old')
                log_file.rename(archive_path)
                rotated.append(str(log_file))

        return rotated

    def tail_all(self, follow: bool = False):
        """Tail all kernel logs (like tail -f)"""
        log_files = self.get_log_files()

        if not log_files:
            print("No log files found")
            return

        # Open all files
        file_handles = {}
        for log_file in log_files:
            try:
                f = open(log_file, 'r')
                f.seek(0, 2)  # Go to end
                file_handles[log_file] = f
            except IOError:
                pass

        print(f"Tailing {len(file_handles)} log files...")
        print("-" * 80)

        try:
            while True:
                had_output = False

                for log_file, f in file_handles.items():
                    line = f.readline()
                    if line:
                        kernel_id = log_file.stem
                        # Color-code by log level
                        parsed = self.parse_log_line(line.strip())

                        if parsed['level'] == 'ERROR':
                            prefix = f"\033[91m[{kernel_id}]\033[0m"  # Red
                        elif parsed['level'] == 'WARNING':
                            prefix = f"\033[93m[{kernel_id}]\033[0m"  # Yellow
                        elif parsed['level'] == 'CRITICAL':
                            prefix = f"\033[95m[{kernel_id}]\033[0m"  # Magenta
                        else:
                            prefix = f"\033[96m[{kernel_id}]\033[0m"  # Cyan

                        print(f"{prefix} {line.strip()}")
                        had_output = True

                if not follow:
                    break

                if not had_output:
                    time.sleep(0.1)

        except KeyboardInterrupt:
            print("\nStopped tailing logs")
        finally:
            for f in file_handles.values():
                f.close()

    def export_logs(self, output_file: str, format: str = "json"):
        """Export aggregated logs"""
        aggregated = self.aggregate_logs(tail_lines=1000)

        if format == "json":
            with open(output_file, 'w') as f:
                json.dump(aggregated, f, indent=2)
        elif format == "text":
            with open(output_file, 'w') as f:
                for kernel_id, data in aggregated['kernels'].items():
                    f.write(f"\n{'='*60}\n")
                    f.write(f"Kernel: {kernel_id} (Port: {data['port']})\n")
                    f.write(f"Errors: {data['errors']} | Warnings: {data['warnings']}\n")
                    f.write(f"{'='*60}\n")

                    for log in data['recent_logs']:
                        f.write(f"[{log['level']}] {log['message']}\n")

        print(f"Logs exported to {output_file}")


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Log Aggregator for LLMSpell Fleet")
    subparsers = parser.add_subparsers(dest='command', help='Commands')

    # Tail command
    tail_parser = subparsers.add_parser('tail', help='Tail all kernel logs')
    tail_parser.add_argument('-f', '--follow', action='store_true', help='Follow log output')
    tail_parser.add_argument('-n', '--lines', type=int, default=10, help='Number of lines to show')

    # Search command
    search_parser = subparsers.add_parser('search', help='Search logs')
    search_parser.add_argument('pattern', help='Search pattern (regex)')
    search_parser.add_argument('--kernel', help='Search specific kernel')
    search_parser.add_argument('-C', '--context', type=int, default=2, help='Context lines')

    # Aggregate command
    agg_parser = subparsers.add_parser('aggregate', help='Show aggregated log summary')
    agg_parser.add_argument('-n', '--lines', type=int, default=100, help='Lines to analyze per kernel')

    # Monitor command
    mon_parser = subparsers.add_parser('monitor', help='Monitor error rates')
    mon_parser.add_argument('--continuous', action='store_true', help='Continuous monitoring')
    mon_parser.add_argument('--interval', type=int, default=5, help='Check interval (seconds)')

    # Export command
    exp_parser = subparsers.add_parser('export', help='Export logs')
    exp_parser.add_argument('output', help='Output file')
    exp_parser.add_argument('--format', choices=['json', 'text'], default='json')

    # Rotate command
    subparsers.add_parser('rotate', help='Rotate old log files')

    args = parser.parse_args()

    aggregator = LogAggregator()

    if args.command == 'tail':
        if args.follow:
            aggregator.tail_all(follow=True)
        else:
            # Show last N lines from each log
            aggregated = aggregator.aggregate_logs(tail_lines=args.lines)
            for kernel_id, data in aggregated['kernels'].items():
                print(f"\n{'='*60}")
                print(f"Kernel: {kernel_id} (Port: {data['port']})")
                print(f"{'='*60}")
                for log in data['recent_logs']:
                    print(f"[{log['level']}] {log['message']}")

    elif args.command == 'search':
        results = aggregator.search_logs(args.pattern, args.kernel, args.context)
        for result in results:
            print(f"\n{result['file']}:{result['line_number']}")
            print(f"Kernel: {result['kernel_id']}")
            print("Context:")
            for line in result['context']:
                print(f"  {line}")
            print()

    elif args.command == 'aggregate':
        aggregated = aggregator.aggregate_logs(tail_lines=args.lines)
        print(json.dumps(aggregated, indent=2))

    elif args.command == 'monitor':
        if args.continuous:
            try:
                while True:
                    result = aggregator.monitor_errors()
                    print(f"\n{datetime.now().isoformat()}")
                    print(f"Errors: {result['total_errors']} | Warnings: {result['total_warnings']}")
                    if result['alerts']:
                        for alert in result['alerts']:
                            print(f"  🚨 {alert}")
                    time.sleep(args.interval)
            except KeyboardInterrupt:
                print("\nMonitoring stopped")
        else:
            result = aggregator.monitor_errors()
            print(json.dumps(result, indent=2))

    elif args.command == 'export':
        aggregator.export_logs(args.output, args.format)

    elif args.command == 'rotate':
        rotated = aggregator.rotate_logs()
        print(f"Rotated {len(rotated)} log files")
        for file in rotated:
            print(f"  - {file}")

    else:
        parser.print_help()


if __name__ == "__main__":
    main()
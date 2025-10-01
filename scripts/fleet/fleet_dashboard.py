#!/usr/bin/env python3
"""
Fleet Dashboard - Terminal-based monitoring dashboard for LLMSpell kernel fleet
"""

import json
import time
import sys
import os
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any

import psutil

# Try to import optional dependencies
try:
    from rich.console import Console
    from rich.table import Table
    from rich.layout import Layout
    from rich.panel import Panel
    from rich.live import Live
    from rich.text import Text
    RICH_AVAILABLE = True
except ImportError:
    RICH_AVAILABLE = False

class FleetDashboard:
    """Fleet monitoring dashboard"""

    def __init__(self, fleet_dir: Path = None, refresh_interval: int = 5):
        self.fleet_dir = fleet_dir or Path.home() / ".llmspell" / "fleet"
        self.registry_file = self.fleet_dir / "registry.json"
        self.refresh_interval = refresh_interval
        self.alert_thresholds = {
            "memory_mb": 1000,      # Alert if kernel uses > 1GB
            "cpu_percent": 80,      # Alert if CPU > 80%
            "connections": 100,     # Alert if connections > 100
            "uptime_hours": 24     # Alert if uptime > 24 hours
        }

    def load_registry(self) -> Dict:
        """Load kernel registry"""
        if not self.registry_file.exists():
            return {"kernels": []}

        with open(self.registry_file) as f:
            return json.load(f)

    def collect_metrics(self) -> Dict[str, Any]:
        """Collect comprehensive fleet metrics"""
        registry = self.load_registry()

        metrics = {
            "timestamp": datetime.now().isoformat(),
            "kernels": [],
            "summary": {
                "total": len(registry.get("kernels", [])),
                "running": 0,
                "dead": 0,
                "total_memory_mb": 0,
                "total_cpu_percent": 0,
                "alerts": []
            }
        }

        for kernel in registry.get("kernels", []):
            kernel_info = {
                "id": kernel["id"],
                "port": kernel["port"],
                "language": kernel.get("language", "unknown"),
                "status": "unknown"
            }

            try:
                process = psutil.Process(kernel["pid"])

                # Collect process metrics
                mem_info = process.memory_info()
                memory_mb = mem_info.rss / 1024 / 1024
                cpu_percent = process.cpu_percent(interval=0.1)
                uptime_seconds = time.time() - process.create_time()
                uptime_hours = uptime_seconds / 3600

                kernel_info.update({
                    "status": "running",
                    "pid": kernel["pid"],
                    "memory_mb": round(memory_mb, 2),
                    "memory_percent": round(process.memory_percent(), 2),
                    "cpu_percent": round(cpu_percent, 2),
                    "threads": process.num_threads(),
                    "connections": len(process.connections()),
                    "uptime_hours": round(uptime_hours, 2),
                    "nice": process.nice()
                })

                # Update summary
                metrics["summary"]["running"] += 1
                metrics["summary"]["total_memory_mb"] += memory_mb
                metrics["summary"]["total_cpu_percent"] += cpu_percent

                # Check alert thresholds
                if memory_mb > self.alert_thresholds["memory_mb"]:
                    metrics["summary"]["alerts"].append(
                        f"High memory: {kernel['id']} using {memory_mb:.0f}MB"
                    )
                if cpu_percent > self.alert_thresholds["cpu_percent"]:
                    metrics["summary"]["alerts"].append(
                        f"High CPU: {kernel['id']} using {cpu_percent:.0f}%"
                    )
                if uptime_hours > self.alert_thresholds["uptime_hours"]:
                    metrics["summary"]["alerts"].append(
                        f"Long uptime: {kernel['id']} running {uptime_hours:.0f}h"
                    )

            except psutil.NoSuchProcess:
                kernel_info["status"] = "dead"
                metrics["summary"]["dead"] += 1
            except (psutil.AccessDenied, Exception) as e:
                kernel_info["status"] = "error"
                kernel_info["error"] = str(e)

            metrics["kernels"].append(kernel_info)

        # Round summary values
        if metrics["summary"]["running"] > 0:
            metrics["summary"]["avg_memory_mb"] = round(
                metrics["summary"]["total_memory_mb"] / metrics["summary"]["running"], 2
            )
            metrics["summary"]["avg_cpu_percent"] = round(
                metrics["summary"]["total_cpu_percent"] / metrics["summary"]["running"], 2
            )

        metrics["summary"]["total_memory_mb"] = round(metrics["summary"]["total_memory_mb"], 2)
        metrics["summary"]["total_cpu_percent"] = round(metrics["summary"]["total_cpu_percent"], 2)

        return metrics

    def format_simple_dashboard(self, metrics: Dict) -> str:
        """Format dashboard in simple text format"""
        lines = []

        # Header
        lines.append("=" * 80)
        lines.append(f"LLMSpell Fleet Dashboard - {metrics['timestamp']}")
        lines.append("=" * 80)

        # Summary
        summary = metrics["summary"]
        lines.append("\nFLEET SUMMARY:")
        lines.append(f"  Total Kernels: {summary['total']}")
        lines.append(f"  Running: {summary['running']} | Dead: {summary['dead']}")
        lines.append(f"  Total Memory: {summary['total_memory_mb']:.1f} MB")
        lines.append(f"  Total CPU: {summary['total_cpu_percent']:.1f}%")

        if summary.get('avg_memory_mb'):
            lines.append(f"  Avg Memory: {summary['avg_memory_mb']:.1f} MB")
            lines.append(f"  Avg CPU: {summary['avg_cpu_percent']:.1f}%")

        # Alerts
        if summary["alerts"]:
            lines.append("\nALERTS:")
            for alert in summary["alerts"]:
                lines.append(f"  ⚠️  {alert}")

        # Kernel details
        lines.append("\nKERNEL DETAILS:")
        lines.append("-" * 80)
        lines.append(f"{'ID':12} {'Port':6} {'Lang':8} {'Status':8} {'Memory':10} {'CPU':8} {'Uptime':10}")
        lines.append("-" * 80)

        for kernel in metrics["kernels"]:
            if kernel["status"] == "running":
                line = f"{kernel['id'][:12]:12} "
                line += f"{kernel['port']:6} "
                line += f"{kernel['language'][:8]:8} "
                line += f"{'✓ RUN':8} "
                line += f"{kernel['memory_mb']:8.1f}MB "
                line += f"{kernel['cpu_percent']:6.1f}% "
                line += f"{kernel['uptime_hours']:8.1f}h"
                lines.append(line)
            elif kernel["status"] == "dead":
                line = f"{kernel['id'][:12]:12} "
                line += f"{kernel['port']:6} "
                line += f"{kernel['language'][:8]:8} "
                line += f"{'✗ DEAD':8}"
                lines.append(line)

        lines.append("-" * 80)

        # Resource usage bar charts
        lines.append("\nRESOURCE USAGE:")
        for kernel in metrics["kernels"]:
            if kernel["status"] == "running":
                # Memory bar
                mem_percent = min(kernel.get("memory_percent", 0), 100)
                bar_length = int(mem_percent / 2)
                mem_bar = "█" * bar_length + "░" * (50 - bar_length)

                lines.append(f"  {kernel['id'][:8]:8} Mem: [{mem_bar}] {mem_percent:.1f}%")

                # CPU bar
                cpu_percent = min(kernel.get("cpu_percent", 0), 100)
                bar_length = int(cpu_percent / 2)
                cpu_bar = "█" * bar_length + "░" * (50 - bar_length)

                lines.append(f"  {' ':8} CPU: [{cpu_bar}] {cpu_percent:.1f}%")
                lines.append("")

        return "\n".join(lines)

    def format_rich_dashboard(self, metrics: Dict):
        """Format dashboard using rich library"""
        if not RICH_AVAILABLE:
            return self.format_simple_dashboard(metrics)

        # Create main table
        table = Table(title=f"LLMSpell Fleet Dashboard - {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")

        table.add_column("Kernel ID", style="cyan")
        table.add_column("Port", justify="center")
        table.add_column("Language", justify="center")
        table.add_column("Status", justify="center")
        table.add_column("Memory", justify="right", style="yellow")
        table.add_column("CPU %", justify="right", style="green")
        table.add_column("Connections", justify="center")
        table.add_column("Uptime", justify="right")

        for kernel in metrics["kernels"]:
            if kernel["status"] == "running":
                status = Text("✓ Running", style="green")
                memory = f"{kernel['memory_mb']:.1f} MB"
                cpu = f"{kernel['cpu_percent']:.1f}%"
                connections = str(kernel.get('connections', 0))
                uptime = f"{kernel['uptime_hours']:.1f}h"

                # Add style based on thresholds
                if kernel['memory_mb'] > self.alert_thresholds['memory_mb']:
                    memory = Text(memory, style="bold red")
                if kernel['cpu_percent'] > self.alert_thresholds['cpu_percent']:
                    cpu = Text(cpu, style="bold red")

                table.add_row(
                    kernel['id'][:12],
                    str(kernel['port']),
                    kernel['language'],
                    status,
                    memory,
                    cpu,
                    connections,
                    uptime
                )
            else:
                status = Text("✗ Dead", style="red")
                table.add_row(
                    kernel['id'][:12],
                    str(kernel['port']),
                    kernel['language'],
                    status,
                    "-", "-", "-", "-"
                )

        return table

    def run_dashboard(self, once: bool = False):
        """Run the dashboard"""
        if once:
            # Single run mode
            metrics = self.collect_metrics()

            if RICH_AVAILABLE:
                console = Console()
                table = self.format_rich_dashboard(metrics)
                console.print(table)

                # Print summary panel
                summary = metrics["summary"]
                summary_text = f"""
[bold]Fleet Summary[/bold]
Total Kernels: {summary['total']} (Running: {summary['running']}, Dead: {summary['dead']})
Total Memory: {summary['total_memory_mb']:.1f} MB
Total CPU: {summary['total_cpu_percent']:.1f}%
"""
                if summary["alerts"]:
                    summary_text += "\n[bold red]Alerts:[/bold red]\n"
                    for alert in summary["alerts"]:
                        summary_text += f"  ⚠️  {alert}\n"

                panel = Panel(summary_text, title="Summary", border_style="blue")
                console.print(panel)
            else:
                print(self.format_simple_dashboard(metrics))
        else:
            # Continuous monitoring mode
            if RICH_AVAILABLE:
                console = Console()
                with Live(console=console, refresh_per_second=1) as live:
                    while True:
                        try:
                            metrics = self.collect_metrics()

                            # Create layout
                            layout = Layout()
                            layout.split_column(
                                Layout(self.format_rich_dashboard(metrics), name="table"),
                                Layout(name="summary", size=10)
                            )

                            # Add summary
                            summary = metrics["summary"]
                            summary_text = f"""
Total: {summary['total']} | Running: {summary['running']} | Dead: {summary['dead']}
Memory: {summary['total_memory_mb']:.1f} MB (Avg: {summary.get('avg_memory_mb', 0):.1f} MB)
CPU: {summary['total_cpu_percent']:.1f}% (Avg: {summary.get('avg_cpu_percent', 0):.1f}%)
"""
                            if summary["alerts"]:
                                summary_text += "\n[bold red]Alerts:[/bold red] "
                                summary_text += " | ".join(summary["alerts"][:3])

                            layout["summary"].update(Panel(summary_text, title="Summary"))

                            live.update(layout)
                            time.sleep(self.refresh_interval)
                        except KeyboardInterrupt:
                            break
            else:
                # Simple continuous mode
                try:
                    while True:
                        os.system('clear' if os.name == 'posix' else 'cls')
                        metrics = self.collect_metrics()
                        print(self.format_simple_dashboard(metrics))
                        print(f"\nRefreshing every {self.refresh_interval} seconds... (Ctrl+C to exit)")
                        time.sleep(self.refresh_interval)
                except KeyboardInterrupt:
                    print("\nDashboard stopped.")

    def export_metrics(self, output_file: str, format: str = "json"):
        """Export metrics to file"""
        metrics = self.collect_metrics()

        if format == "json":
            with open(output_file, 'w') as f:
                json.dump(metrics, f, indent=2)
        elif format == "csv":
            import csv
            with open(output_file, 'w', newline='') as f:
                writer = csv.writer(f)
                writer.writerow(['timestamp', 'kernel_id', 'port', 'status', 'memory_mb', 'cpu_percent', 'uptime_hours'])
                for kernel in metrics['kernels']:
                    writer.writerow([
                        metrics['timestamp'],
                        kernel['id'],
                        kernel['port'],
                        kernel['status'],
                        kernel.get('memory_mb', 0),
                        kernel.get('cpu_percent', 0),
                        kernel.get('uptime_hours', 0)
                    ])

        print(f"Metrics exported to {output_file}")


def main():
    """Main entry point"""
    parser = argparse.ArgumentParser(description="Fleet Dashboard for LLMSpell Kernels")
    parser.add_argument('--once', action='store_true', help='Run once and exit')
    parser.add_argument('--refresh', type=int, default=5, help='Refresh interval in seconds')
    parser.add_argument('--export', help='Export metrics to file')
    parser.add_argument('--format', choices=['json', 'csv'], default='json', help='Export format')
    parser.add_argument('--threshold-memory', type=int, default=1000, help='Memory alert threshold (MB)')
    parser.add_argument('--threshold-cpu', type=int, default=80, help='CPU alert threshold (%)')

    args = parser.parse_args()

    dashboard = FleetDashboard(refresh_interval=args.refresh)

    # Set custom thresholds
    dashboard.alert_thresholds['memory_mb'] = args.threshold_memory
    dashboard.alert_thresholds['cpu_percent'] = args.threshold_cpu

    if args.export:
        dashboard.export_metrics(args.export, args.format)
    else:
        dashboard.run_dashboard(once=args.once)


if __name__ == "__main__":
    main()
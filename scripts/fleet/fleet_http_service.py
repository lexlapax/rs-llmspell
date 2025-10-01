#!/usr/bin/env python3
"""
Fleet HTTP Service - REST API for kernel discovery and management
"""

from flask import Flask, jsonify, request
from fleet_manager import FleetManager
import argparse

app = Flask(__name__)
fleet = FleetManager()

@app.route('/health', methods=['GET'])
def health():
    """Health check endpoint"""
    return jsonify({"status": "healthy"}), 200

@app.route('/kernels', methods=['GET'])
def list_kernels():
    """List all kernels"""
    fleet.cleanup_dead_kernels()
    kernels = []
    for kernel in fleet.registry["kernels"]:
        kernels.append({
            "id": kernel["id"],
            "port": kernel["port"],
            "language": kernel["language"],
            "config": kernel["config"],
            "status": kernel.get("status", "unknown"),
            "connection_file": kernel["connection_file"]
        })
    return jsonify({"kernels": kernels}), 200

@app.route('/kernels/<kernel_id>', methods=['GET'])
def get_kernel(kernel_id):
    """Get specific kernel info"""
    kernel = next((k for k in fleet.registry["kernels"]
                   if k["id"] == kernel_id), None)
    if kernel:
        return jsonify(kernel), 200
    return jsonify({"error": "Kernel not found"}), 404

@app.route('/kernels', methods=['POST'])
def spawn_kernel():
    """Spawn a new kernel"""
    data = request.json or {}
    kernel = fleet.spawn_kernel(
        config_file=data.get("config", "default.toml"),
        language=data.get("language", "lua"),
        memory_limit=data.get("memory_limit"),
        cpu_limit=data.get("cpu_limit"),
        env_vars=data.get("env_vars")
    )
    if kernel:
        return jsonify(kernel), 201
    return jsonify({"error": "Failed to spawn kernel"}), 500

@app.route('/kernels/<kernel_id>', methods=['DELETE'])
def stop_kernel(kernel_id):
    """Stop a kernel"""
    if fleet.stop_kernel(kernel_id):
        return jsonify({"message": f"Kernel {kernel_id} stopped"}), 200
    return jsonify({"error": "Failed to stop kernel"}), 500

@app.route('/find', methods=['POST'])
def find_or_create():
    """Find matching kernel or create new one"""
    requirements = request.json or {}
    kernel = fleet.find_or_create_kernel(requirements)
    if kernel:
        return jsonify(kernel), 200
    return jsonify({"error": "Failed to find or create kernel"}), 500

@app.route('/metrics', methods=['GET'])
def get_metrics():
    """Get fleet metrics - supports both JSON and Prometheus formats"""
    # Check for Prometheus format request
    if request.headers.get('Accept') == 'text/plain' or request.args.get('format') == 'prometheus':
        return get_prometheus_metrics()

    # Default to JSON format
    metrics = fleet.get_metrics()
    return jsonify(metrics), 200

@app.route('/metrics/prometheus', methods=['GET'])
def get_prometheus_metrics():
    """Export metrics in Prometheus format"""
    fleet.cleanup_dead_kernels()
    metrics = fleet.get_metrics()

    # Build Prometheus format output
    lines = []

    # Help and type annotations
    lines.extend([
        "# HELP llmspell_kernels_total Total number of kernels in registry",
        "# TYPE llmspell_kernels_total gauge",
        f"llmspell_kernels_total {metrics['total_kernels']}",
        "",
        "# HELP llmspell_kernels_active Number of active running kernels",
        "# TYPE llmspell_kernels_active gauge",
        f"llmspell_kernels_active {metrics['aggregated']['active_kernels']}",
        "",
        "# HELP llmspell_kernels_dead Number of dead kernels",
        "# TYPE llmspell_kernels_dead gauge",
        f"llmspell_kernels_dead {metrics['aggregated']['dead_kernels']}",
        "",
        "# HELP llmspell_memory_mb_total Total memory usage in MB",
        "# TYPE llmspell_memory_mb_total gauge",
        f"llmspell_memory_mb_total {metrics['aggregated']['total_memory_mb']}",
        "",
        "# HELP llmspell_cpu_percent_total Total CPU usage percentage",
        "# TYPE llmspell_cpu_percent_total gauge",
        f"llmspell_cpu_percent_total {metrics['aggregated']['total_cpu_percent']}",
        "",
        "# HELP llmspell_connections_total Total number of connections",
        "# TYPE llmspell_connections_total gauge",
        f"llmspell_connections_total {metrics['aggregated']['total_connections']}",
        "",
        "# HELP llmspell_threads_total Total number of threads",
        "# TYPE llmspell_threads_total gauge",
        f"llmspell_threads_total {metrics['aggregated']['total_threads']}",
        ""
    ])

    # Per-kernel metrics with labels
    lines.extend([
        "# HELP llmspell_kernel_memory_mb Memory usage per kernel in MB",
        "# TYPE llmspell_kernel_memory_mb gauge"
    ])

    for kernel in metrics['kernels']:
        if kernel.get('status') == 'running':
            kernel_id = kernel['id']
            port = kernel['port']
            language = kernel['language']

            # Memory metric
            lines.append(
                f'llmspell_kernel_memory_mb{{kernel_id="{kernel_id}",port="{port}",language="{language}"}} {kernel.get("memory_mb", 0):.2f}'
            )

    lines.append("")
    lines.extend([
        "# HELP llmspell_kernel_cpu_percent CPU usage per kernel",
        "# TYPE llmspell_kernel_cpu_percent gauge"
    ])

    for kernel in metrics['kernels']:
        if kernel.get('status') == 'running':
            kernel_id = kernel['id']
            port = kernel['port']
            language = kernel['language']

            # CPU metric
            lines.append(
                f'llmspell_kernel_cpu_percent{{kernel_id="{kernel_id}",port="{port}",language="{language}"}} {kernel.get("cpu_percent", 0):.2f}'
            )

    lines.append("")
    lines.extend([
        "# HELP llmspell_kernel_uptime_seconds Uptime per kernel in seconds",
        "# TYPE llmspell_kernel_uptime_seconds counter"
    ])

    for kernel in metrics['kernels']:
        if kernel.get('status') == 'running':
            kernel_id = kernel['id']
            port = kernel['port']
            language = kernel['language']

            # Uptime metric
            lines.append(
                f'llmspell_kernel_uptime_seconds{{kernel_id="{kernel_id}",port="{port}",language="{language}"}} {kernel.get("uptime_seconds", 0):.0f}'
            )

    lines.append("")
    lines.extend([
        "# HELP llmspell_kernel_connections Number of connections per kernel",
        "# TYPE llmspell_kernel_connections gauge"
    ])

    for kernel in metrics['kernels']:
        if kernel.get('status') == 'running':
            kernel_id = kernel['id']
            port = kernel['port']
            language = kernel['language']

            # Connections metric
            lines.append(
                f'llmspell_kernel_connections{{kernel_id="{kernel_id}",port="{port}",language="{language}"}} {kernel.get("connections", 0)}'
            )

    # Return Prometheus format text
    response = '\n'.join(lines)
    return response, 200, {'Content-Type': 'text/plain; version=0.0.4'}

@app.route('/registry', methods=['GET'])
def get_registry():
    """Get raw registry (for debugging)"""
    return jsonify(fleet.registry), 200

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Fleet HTTP Service")
    parser.add_argument('--port', type=int, default=9550,
                        help='HTTP service port')
    parser.add_argument('--host', default='127.0.0.1',
                        help='HTTP service host')
    args = parser.parse_args()

    print(f"Starting Fleet HTTP Service on {args.host}:{args.port}")
    app.run(host=args.host, port=args.port, debug=False)
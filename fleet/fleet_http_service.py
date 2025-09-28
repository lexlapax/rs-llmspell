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
    """Get fleet metrics"""
    metrics = fleet.get_metrics()
    return jsonify(metrics), 200

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
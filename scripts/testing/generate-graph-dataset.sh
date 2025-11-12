#!/bin/bash
# Generate synthetic graph dataset for performance testing (Task 13c.2.8.6)
#
# Usage:
#   ./scripts/testing/generate-graph-dataset.sh [output_dir]
#
# Output:
#   - entities.json (100K entities)
#   - relationships.json (~1M relationships)
#   - dataset-summary.txt
#
# Requirements:
#   - cargo (Rust toolchain)
#   - rust-script (install via: cargo install rust-script)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OUTPUT_DIR="${1:-$SCRIPT_DIR/../../benchmarks/graph-dataset}"

echo "=== Synthetic Graph Dataset Generator ==="
echo "Output directory: $OUTPUT_DIR"
echo

# Create output directory
mkdir -p "$OUTPUT_DIR"
cd "$OUTPUT_DIR"

# Check if rust-script is installed
if ! command -v rust-script &> /dev/null; then
    echo "Error: rust-script not found"
    echo "Install with: cargo install rust-script"
    exit 1
fi

# Run generator
echo "Running generator (this may take 2-3 minutes)..."
time rust-script "$SCRIPT_DIR/generate-graph-dataset.rs"

echo
echo "=== Generation Complete ==="
echo "Files created in: $OUTPUT_DIR"
ls -lh entities.json relationships.json dataset-summary.txt
echo
echo "Summary:"
cat dataset-summary.txt

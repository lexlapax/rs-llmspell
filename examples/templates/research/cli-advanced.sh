#!/bin/bash
# Advanced Research Assistant Template Example
# This demonstrates all available parameters with custom configuration

# Create output directory for artifacts
OUTPUT_DIR="./research_output"
mkdir -p "$OUTPUT_DIR"

# Run the research assistant with all custom parameters
# - Custom topic
# - Limited to 5 sources (faster execution)
# - Using smaller/faster model (llama3.2:1b)
# - JSON output format for programmatic processing
# - Citations disabled for cleaner output
# - Save artifacts to output directory

llmspell template exec research-assistant \
  --param topic="Machine learning model interpretability techniques" \
  --param max_sources=5 \
  --param model="ollama/llama3.2:1b" \
  --param output_format="json" \
  --param include_citations=false \
  --output "$OUTPUT_DIR"

echo ""
echo "✓ Research assistant executed with custom parameters"
echo "  - Topic: Machine learning model interpretability techniques"
echo "  - Max sources: 5 (custom)"
echo "  - Model: ollama/llama3.2:1b (faster, smaller model)"
echo "  - Output format: JSON (programmatic)"
echo "  - Citations: disabled (cleaner output)"
echo "  - Artifacts saved to: $OUTPUT_DIR/"
echo ""
echo "Generated artifacts:"
ls -lh "$OUTPUT_DIR/" 2>/dev/null || echo "  (no artifacts yet - requires full infrastructure)"

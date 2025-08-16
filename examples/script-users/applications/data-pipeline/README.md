# Production Data Pipeline v2.0

A production-ready ETL pipeline demonstrating llmspell's nested workflow architecture with LLM-powered analysis.

## Overview

This application showcases proper component composition using:
- **Sequential Workflow**: Main orchestration
- **Parallel Workflows**: Multi-source extraction and concurrent analysis
- **Loop Workflow**: Batch transformation processing
- **5 LLM Agents**: Data enrichment, quality analysis, anomaly detection, pattern recognition, and report generation
- **State Management**: Checkpointing and recovery
- **Production Features**: Error handling, monitoring, notifications

## Architecture

```yaml
Main Pipeline (Sequential)
├── Extract Phase (Parallel)
│   ├── Database extraction
│   ├── API extraction
│   └── File extraction
├── Transform Phase (Loop)
│   ├── Batch validation
│   ├── Data cleaning
│   └── LLM enrichment
├── Analysis Phase (Parallel)
│   ├── Quality analysis (LLM)
│   ├── Anomaly detection (LLM)
│   └── Pattern recognition (LLM)
└── Load Phase (Sequential)
    ├── Save to database
    ├── Generate report (LLM)
    └── Send notifications
```

## Prerequisites

### Required API Keys
Set at least one of these environment variables:
```bash
export OPENAI_API_KEY="your-openai-api-key"
export ANTHROPIC_API_KEY="your-anthropic-api-key"
```

### Configuration File
Use the provided `config.toml`:
```bash
export LLMSPELL_CONFIG=examples/script-users/applications/data-pipeline/config.toml
```

## Quick Start

### 1. Basic Run (with mock data)
```bash
# From llmspell root directory
./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
```

### 2. With Configuration
```bash
# Use the application config
LLMSPELL_CONFIG=examples/script-users/applications/data-pipeline/config.toml \
./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
```

### 3. With API Keys (full LLM features)
```bash
# Set API keys for full functionality
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."

# Run with config
LLMSPELL_CONFIG=examples/script-users/applications/data-pipeline/config.toml \
./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
```

## Features Demonstrated

### 1. Nested Workflows
- Main sequential workflow orchestrates 4 phases
- Each phase uses appropriate workflow type (Parallel, Loop, Sequential)
- Workflows can be nested arbitrarily deep

### 2. LLM Integration (5 Agents)
- **Data Enricher** (GPT-3.5-turbo): Adds contextual information
- **Quality Analyzer** (GPT-4): Identifies data quality issues
- **Anomaly Detector** (GPT-4): Finds outliers and anomalies
- **Pattern Finder** (Claude-3-haiku): Discovers patterns and trends
- **Report Generator** (Claude-3-sonnet): Creates executive reports

### 3. Production Features
- **Multi-source extraction**: Database, API, and file sources in parallel
- **Batch processing**: Loop workflow processes data in chunks
- **Error handling**: Validation and data cleaning
- **State persistence**: Checkpointing for recovery
- **Monitoring**: Metrics and progress tracking
- **Notifications**: Pipeline completion alerts

## Configuration Options

Edit `config.toml` to customize:

```toml
[pipeline]
batch_size = 100              # Total records to process
batch_chunk_size = 10         # Records per batch in Loop workflow
checkpoint_interval = 5       # Batches between checkpoints

[models]
enricher = "openai/gpt-3.5-turbo"
quality = "openai/gpt-4o-mini"
anomaly = "openai/gpt-4o-mini"
patterns = "anthropic/claude-3-haiku-20240307"
report = "anthropic/claude-3-sonnet-20240229"

[sources]
database_url = "postgresql://localhost/data"
api_endpoint = "https://api.example.com/data"
file_directory = "/data/input"
```

## Output Files

The pipeline generates several output files:
- `/tmp/pipeline_db.json` - Database extraction
- `/tmp/pipeline_api.json` - API extraction
- `/tmp/pipeline_files.json` - File extraction
- `/tmp/pipeline_output.json` - Processed data
- `/tmp/pipeline_report.txt` - Executive report

## Cost Considerations

⚠️ **This application uses REAL LLM APIs that incur costs!**

Estimated costs per run (with default settings):
- Data Enricher: ~$0.01 (100 records @ GPT-3.5-turbo)
- Quality Analyzer: ~$0.02 (GPT-4)
- Anomaly Detector: ~$0.02 (GPT-4)
- Pattern Finder: ~$0.01 (Claude-3-haiku)
- Report Generator: ~$0.03 (Claude-3-sonnet)
- **Total: ~$0.09 per full pipeline run**

### Cost Optimization Tips
1. Use smaller models for development (gpt-3.5-turbo instead of gpt-4)
2. Reduce batch_size for testing
3. Disable specific agents during development
4. Use checkpoint recovery to avoid re-processing

## Testing

Run the test suite:
```bash
./target/debug/llmspell run examples/script-users/applications/data-pipeline/test.lua
```

## Troubleshooting

### No API Keys
Without API keys, the pipeline runs with simulated data and skips LLM analysis:
- ✅ Workflow orchestration works
- ✅ Data extraction and transformation work
- ⚠️ LLM agents show as "skipped"
- ⚠️ No enrichment or analysis performed

### Partial API Keys
With only one provider's API key:
- Agents using available provider work normally
- Agents using unavailable provider are skipped
- Pipeline continues with degraded functionality

### Performance Issues
- Reduce `batch_size` in config
- Increase `batch_chunk_size` for fewer iterations
- Use faster models (gpt-3.5-turbo, claude-3-haiku)

### Recovery from Failure
The pipeline saves checkpoints automatically:
```lua
-- Load and examine last checkpoint
local checkpoint = State.load("pipeline_v2", "last_run")
print(checkpoint.records_processed)
```

## Blueprint Compliance

This implementation follows the blueprint v2.0 architecture:
- ✅ Component composition (Workflows + Agents + Tools)
- ✅ Minimal Lua (only orchestration logic)
- ✅ Production-grade error handling
- ✅ State persistence and recovery
- ✅ Real LLM integration (no mocks)
- ✅ Proper workflow nesting (Sequential → Parallel → Loop)

## Next Steps

1. **Customize for your data**:
   - Modify `generate_sample_data()` to load real data
   - Update validation rules for your schema
   - Adjust batch sizes for your volume

2. **Add integrations**:
   - Replace file operations with real database connections
   - Add webhook notifications
   - Integrate with monitoring systems

3. **Extend analysis**:
   - Add custom analysis agents
   - Implement domain-specific patterns
   - Create specialized reports

## Support

For issues or questions:
- Check the [llmspell documentation](../../README.md)
- Review the [blueprint specification](../blueprint.md)
- See other [example applications](../README.md)
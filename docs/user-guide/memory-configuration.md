# Memory System Configuration

**Version**: 0.13.0 (Phase 13 - Adaptive Memory System)
**Status**: Production Ready
**Last Updated**: October 2025

## Overview

The LLMSpell memory system provides adaptive, long-term memory for AI agents through:
- **Episodic Memory**: Conversation history with temporal context
- **Semantic Memory**: Knowledge extraction and consolidation
- **LLM-Driven Consolidation**: Intelligent summarization and entity extraction
- **Temporal Knowledge Graph**: Bi-temporal relationships (event time + ingestion time)
- **Adaptive Daemon**: Smart scheduling based on activity levels

The memory system is fully integrated with the provider system, allowing centralized LLM configuration management.

## Quick Start

### 1. Using the Memory Profile

The simplest way to enable memory is using the builtin `memory` profile:

```bash
llmspell --profile memory
```

This loads `~/.llmspell/config/memory.toml` (or uses builtin defaults) with:
- Memory system enabled
- Two providers: `default` (general tasks) and `consolidation-llm` (memory operations)
- Daemon with adaptive scheduling
- Production-ready defaults

### 2. Manual Configuration

Add to your `config.toml`:

```toml
# Provider for memory consolidation
[providers.consolidation-llm]
provider_type = "ollama"
default_model = "llama3.2:3b"
temperature = 0.0  # Low temperature for deterministic consolidation
max_tokens = 2000
timeout_seconds = 30
max_retries = 3

# Enable memory system
[runtime.memory]
enabled = true

# Consolidation configuration
[runtime.memory.consolidation]
provider_name = "consolidation-llm"  # Reference provider by name
batch_size = 10
max_concurrent = 3
active_session_threshold_secs = 300

# Daemon configuration
[runtime.memory.daemon]
enabled = true
fast_interval_secs = 30
normal_interval_secs = 300
slow_interval_secs = 600
queue_threshold_fast = 5
queue_threshold_slow = 20
shutdown_max_wait_secs = 30
health_check_interval_secs = 60
```

## Configuration Reference

### Memory Configuration (`runtime.memory`)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | false | Enable/disable memory system |

### Consolidation Configuration (`runtime.memory.consolidation`)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `provider_name` | Option\<String\> | None | Provider for LLM consolidation (falls back to `default_provider`) |
| `batch_size` | usize | 10 | Number of memories to consolidate in single batch |
| `max_concurrent` | usize | 3 | Maximum concurrent consolidation operations |
| `active_session_threshold_secs` | u64 | 300 | Seconds of inactivity before session considered inactive |

**Key Design**: `provider_name` references a provider defined in `[providers.<name>]`, enabling centralized LLM config management. Do NOT specify `model`, `temperature`, or other LLM parameters here - those are configured in the provider.

### Daemon Configuration (`runtime.memory.daemon`)

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `enabled` | bool | true | Enable/disable background consolidation daemon |
| `fast_interval_secs` | u64 | 30 | Consolidation interval when queue is full (high activity) |
| `normal_interval_secs` | u64 | 300 | Default consolidation interval |
| `slow_interval_secs` | u64 | 600 | Consolidation interval when queue is empty (low activity) |
| `queue_threshold_fast` | usize | 5 | Queue size to trigger fast interval |
| `queue_threshold_slow` | usize | 20 | Queue size to trigger normal interval (above this) |
| `shutdown_max_wait_secs` | u64 | 30 | Maximum time to wait for daemon shutdown |
| `health_check_interval_secs` | u64 | 60 | Health check frequency |

**Adaptive Scheduling**: Daemon automatically adjusts consolidation frequency based on queue depth:
- **Fast mode** (30s): Queue ≥ 5 items (high activity)
- **Normal mode** (300s): Queue between threshold_fast and threshold_slow
- **Slow mode** (600s): Queue < 5 items (low activity)

## Provider Integration

### Why Use Providers for Memory?

The memory system uses the provider system for LLM consolidation, providing:

1. **Centralized Configuration**: Define LLM settings once, reference by name
2. **Version Control**: Provider configs tracked in `config.toml`
3. **Easy Rotation**: Change model by updating provider, not consolidation config
4. **Consistency**: Same provider can be used across memory, templates, agents
5. **Testability**: Swap providers for testing without changing consolidation config

### Provider Requirements

Memory consolidation requires a provider with:
- **Low temperature** (0.0-0.2): Ensures deterministic, consistent consolidation
- **Sufficient tokens** (1500-2000): Handles batch consolidation
- **Reasonable timeout** (30-60s): Allows for complex entity extraction

### Example: Dedicated Consolidation Provider

```toml
[providers]
default_provider = "default"

# General-purpose provider (templates, agents)
[providers.default]
provider_type = "ollama"
default_model = "llama3.2:3b"
temperature = 0.7  # Higher temp for creative tasks
max_tokens = 4096

# Memory consolidation provider (low temp for consistency)
[providers.consolidation-llm]
provider_type = "ollama"
default_model = "llama3.2:3b"
temperature = 0.0  # Deterministic consolidation
max_tokens = 2000
timeout_seconds = 30

[runtime.memory.consolidation]
provider_name = "consolidation-llm"  # Reference by name
```

**Why Separate Providers?**
- General tasks benefit from higher temperature (creativity, variety)
- Memory consolidation requires low temperature (consistency, fact extraction)
- Allows using different models (e.g., larger model for consolidation, smaller for general use)

## Use Cases

### 1. Conversational Agents with Long-Term Memory

**Scenario**: Chat agent that remembers user preferences, past conversations

```toml
[runtime.memory]
enabled = true

[runtime.memory.consolidation]
provider_name = "consolidation-llm"
batch_size = 10
active_session_threshold_secs = 1800  # 30 minutes

[runtime.memory.daemon]
enabled = true
fast_interval_secs = 60  # Frequent consolidation during active chat
```

**Benefit**: Agent recalls context from previous sessions, builds user profile over time

### 2. Knowledge Accumulation

**Scenario**: Research assistant that builds knowledge base from multiple sources

```toml
[runtime.memory]
enabled = true

[runtime.memory.consolidation]
provider_name = "consolidation-llm"
batch_size = 20  # Larger batches for document processing
max_concurrent = 5  # Parallel consolidation

[runtime.memory.daemon]
slow_interval_secs = 1800  # Less frequent consolidation for batch workloads
```

**Benefit**: Extracts entities and relationships from documents, builds temporal knowledge graph

### 3. Long-Running Agents

**Scenario**: Monitoring agent that tracks system state over days/weeks

```toml
[runtime.memory]
enabled = true

[runtime.memory.consolidation]
provider_name = "consolidation-llm"
batch_size = 50  # Large batches for high-volume data
active_session_threshold_secs = 86400  # 24 hours

[runtime.memory.daemon]
enabled = true
normal_interval_secs = 600  # 10-minute consolidation
```

**Benefit**: Maintains compact memory of system events, detects patterns over time

## Performance Tuning

### Fast Iteration (Development)

**Goal**: Quick feedback during development

```toml
[runtime.memory.consolidation]
batch_size = 5  # Small batches
max_concurrent = 1  # Sequential processing

[runtime.memory.daemon]
fast_interval_secs = 10  # Aggressive consolidation
```

**Trade-off**: Higher CPU usage, faster consolidation

### Memory-Constrained Environments

**Goal**: Minimize memory footprint

```toml
[runtime.memory.consolidation]
batch_size = 50  # Large batches reduce memory overhead
max_concurrent = 1  # One batch in memory at a time

[runtime.memory.daemon]
slow_interval_secs = 1200  # Infrequent consolidation
```

**Trade-off**: Slower consolidation, lower memory usage

### High Throughput

**Goal**: Maximum consolidation throughput

```toml
[runtime.memory.consolidation]
batch_size = 20  # Balanced batch size
max_concurrent = 8  # High parallelism

[runtime.memory.daemon]
fast_interval_secs = 15
normal_interval_secs = 120
queue_threshold_fast = 10
```

**Trade-off**: Higher CPU/memory usage, faster processing

### Production Defaults (Recommended)

**Goal**: Balanced performance, reliability, cost

```toml
[runtime.memory.consolidation]
provider_name = "consolidation-llm"
batch_size = 10
max_concurrent = 3
active_session_threshold_secs = 300

[runtime.memory.daemon]
enabled = true
fast_interval_secs = 30
normal_interval_secs = 300
slow_interval_secs = 600
queue_threshold_fast = 5
queue_threshold_slow = 20
```

**Why**: Tested defaults balance CPU usage, latency, and LLM costs

## Troubleshooting

### Problem: Memory not consolidating

**Symptoms**: Daemon running but no consolidation happening

**Solutions**:
1. Check `runtime.memory.enabled = true`
2. Verify provider exists: `llmspell config show | grep consolidation-llm`
3. Check daemon status: `llmspell memory status`
4. Review logs: `tail -f ~/.llmspell/logs/daemon.log`

### Problem: High CPU usage

**Symptoms**: Daemon consuming excessive CPU

**Solutions**:
1. Increase `normal_interval_secs` (reduce consolidation frequency)
2. Decrease `max_concurrent` (reduce parallelism)
3. Increase `batch_size` (fewer consolidation calls)
4. Check queue depth: `llmspell memory queue-depth`

### Problem: Consolidation too slow

**Symptoms**: Large backlog of unconsolidated memories

**Solutions**:
1. Increase `max_concurrent` (more parallelism)
2. Decrease `fast_interval_secs` (more frequent consolidation)
3. Lower `queue_threshold_fast` (trigger fast mode earlier)
4. Use faster LLM provider for consolidation

### Problem: Provider not found

**Symptoms**: Error: "provider 'consolidation-llm' not found"

**Solutions**:
1. Verify provider defined in `config.toml`:
   ```bash
   llmspell config show | grep -A 5 "\\[providers.consolidation-llm\\]"
   ```
2. Check spelling of `provider_name` in `consolidation` config
3. Ensure provider is not commented out in config
4. Use `default` provider as fallback:
   ```toml
   provider_name = "default"
   ```

### Problem: Inconsistent consolidation results

**Symptoms**: Different summaries for same input

**Solutions**:
1. Lower provider temperature: `temperature = 0.0`
2. Use deterministic model (avoid sampling-based models)
3. Check provider config: `llmspell config show providers.consolidation-llm`
4. Consider switching to specialized summarization model

## Environment Variables

Override memory configuration via environment variables:

```bash
# Enable/disable memory
export LLMSPELL_MEMORY_ENABLED=true

# Consolidation settings
export LLMSPELL_MEMORY_CONSOLIDATION_PROVIDER_NAME=consolidation-llm
export LLMSPELL_MEMORY_CONSOLIDATION_BATCH_SIZE=10
export LLMSPELL_MEMORY_CONSOLIDATION_MAX_CONCURRENT=3
export LLMSPELL_MEMORY_CONSOLIDATION_ACTIVE_SESSION_THRESHOLD_SECS=300

# Daemon settings
export LLMSPELL_MEMORY_DAEMON_ENABLED=true
export LLMSPELL_MEMORY_DAEMON_FAST_INTERVAL_SECS=30
export LLMSPELL_MEMORY_DAEMON_NORMAL_INTERVAL_SECS=300
export LLMSPELL_MEMORY_DAEMON_SLOW_INTERVAL_SECS=600
export LLMSPELL_MEMORY_DAEMON_QUEUE_THRESHOLD_FAST=5
export LLMSPELL_MEMORY_DAEMON_QUEUE_THRESHOLD_SLOW=20
```

**Precedence**: Environment variables > config.toml > builtin defaults

## Related Documentation

- [Provider Usage Best Practices](provider-best-practices.md) - When to use `provider_name` vs `model`
- [Configuration Guide](configuration.md) - Full configuration reference
- [Performance Tuning](performance-tuning.md) - System-wide optimization
- [Troubleshooting](troubleshooting.md) - Common issues and solutions

## Next Steps

1. **Enable memory**: Choose `memory` profile or manual config
2. **Configure provider**: Create dedicated consolidation provider with low temperature
3. **Tune daemon**: Adjust intervals based on workload (fast iteration vs production)
4. **Monitor**: Check logs and queue depth to verify consolidation working
5. **Optimize**: Adjust batch_size, max_concurrent based on performance needs

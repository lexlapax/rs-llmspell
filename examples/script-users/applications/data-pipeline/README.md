# Production Data Pipeline Application

## Overview

A production-ready data processing pipeline with enterprise features including monitoring, failure recovery, auto-scaling, and real-time alerting. This application demonstrates how to build robust data processing systems using llmspell.

## Features

### Core Capabilities
- **Batch Processing**: Efficient processing of data in configurable batch sizes
- **Stream Processing**: Real-time data processing with windowing support
- **ETL Operations**: Extract, Transform, Load with validation
- **Data Quality**: Automatic validation and quality checks

### Production Features
- **Monitoring & Alerting**
  - Real-time metrics collection
  - Configurable alert thresholds
  - Multi-channel alerting support
  - Performance dashboards

- **Failure Recovery**
  - Automatic retry with exponential backoff
  - Dead Letter Queue (DLQ) for failed records
  - Checkpoint-based recovery
  - Transaction rollback capabilities

- **Auto-Scaling**
  - Dynamic worker scaling based on load
  - Configurable min/max workers
  - Cooldown periods to prevent thrashing
  - Resource utilization optimization

- **Observability**
  - Detailed metrics and logging
  - Performance profiling
  - Error tracking and analysis
  - Audit trails

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Data Sources  │───▶│   Pipeline      │───▶│   Data Sinks    │
│   • Files       │    │   • Process     │    │   • Database    │
│   • APIs        │    │   • Transform   │    │   • Files       │
│   • Streams     │    │   • Validate    │    │   • APIs        │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
    [Monitoring]           [Recovery]             [Scaling]
    • Metrics              • Checkpoints          • Auto-scale
    • Alerts               • DLQ                  • Load balance
    • Dashboards           • Retry                • Resource mgmt
```

## Configuration

The pipeline is configured through the `config` table in `main.lua`:

```lua
local config = {
    pipeline_name = "ProductionDataPipeline",
    batch_size = 100,
    retry_attempts = 3,
    monitoring = {
        enabled = true,
        metrics_interval = 60,  -- seconds
        alert_thresholds = {
            error_rate = 0.05,      -- 5% error rate
            latency_ms = 5000,      -- 5 second latency
            throughput_min = 10     -- 10 records/minute
        }
    },
    recovery = {
        dead_letter_queue = true,
        checkpoint_interval = 100,  -- records
        rollback_enabled = true
    },
    scaling = {
        auto_scale = true,
        min_workers = 1,
        max_workers = 10,
        scale_up_threshold = 0.8,   -- 80% capacity
        scale_down_threshold = 0.2   -- 20% capacity
    }
}
```

## Usage

### Basic Usage

```bash
# Run the pipeline
./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
```

### With Configuration

```bash
# Use custom configuration
LLMSPELL_CONFIG=examples/script-users/configs/production.toml \
  ./target/debug/llmspell run examples/script-users/applications/data-pipeline/main.lua
```

## Monitoring

### Metrics Collected
- **Throughput**: Records processed per minute
- **Latency**: Average processing time per record
- **Error Rate**: Percentage of failed records
- **Worker Utilization**: Active vs idle workers
- **Queue Depth**: Pending records and DLQ size

### Alert Types
- **HIGH**: Error rate exceeds threshold
- **MEDIUM**: High latency detected
- **LOW**: Low throughput warning

### Dashboard Access
Metrics are exposed for integration with monitoring systems:
- Prometheus endpoint: `/metrics`
- Grafana dashboards available
- Custom metrics via API

## Failure Recovery

### Retry Strategy
1. First failure: Retry after 1 second
2. Second failure: Retry after 2 seconds
3. Third failure: Retry after 4 seconds
4. Final failure: Send to Dead Letter Queue

### Checkpoint Recovery
- Automatic checkpoints every N records (configurable)
- Persisted to durable storage
- Resume from last checkpoint on restart
- Transaction-safe processing

### Dead Letter Queue
- Failed records after max retries
- Manual review and reprocessing
- Audit trail of failures
- Export capabilities

## Scaling

### Auto-Scaling Rules
- **Scale Up**: When load > 80% for 60 seconds
- **Scale Down**: When load < 20% for 60 seconds
- **Cooldown**: 60 seconds between scaling events
- **Limits**: Min 1 worker, Max 10 workers

### Manual Scaling
```lua
-- Add workers
pipeline:add_worker()
pipeline:add_worker()

-- Remove workers
pipeline:remove_worker()
```

## Performance

### Benchmarks
- **Throughput**: 10,000+ records/minute
- **Latency**: < 100ms average
- **Error Recovery**: < 5 seconds
- **Scale Time**: < 10 seconds

### Optimization Tips
1. Adjust batch size based on record size
2. Configure workers based on CPU cores
3. Use checkpoints for large datasets
4. Enable caching for repeated transformations

## Integration

### Data Sources
- File systems (CSV, JSON, XML)
- Databases (PostgreSQL, MySQL, MongoDB)
- Message queues (Kafka, RabbitMQ)
- APIs (REST, GraphQL)

### Data Sinks
- Databases with transaction support
- Data warehouses (Snowflake, BigQuery)
- Object storage (S3, GCS)
- Analytics platforms

### Monitoring Integration
- **Prometheus**: Metrics export
- **Grafana**: Pre-built dashboards
- **PagerDuty**: Alert routing
- **Slack**: Notifications

## Deployment

### Docker
```dockerfile
FROM llmspell:latest
COPY ./data-pipeline /app
CMD ["llmspell", "run", "/app/main.lua"]
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: data-pipeline
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: pipeline
        image: llmspell-pipeline:latest
        resources:
          requests:
            memory: "256Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "2000m"
```

## Troubleshooting

### Common Issues

1. **High Error Rate**
   - Check data quality
   - Review transformation logic
   - Increase retry attempts
   - Examine DLQ records

2. **Low Throughput**
   - Increase batch size
   - Add more workers
   - Check network latency
   - Optimize transformations

3. **Memory Issues**
   - Reduce batch size
   - Enable streaming mode
   - Configure memory limits
   - Use lazy loading

4. **Checkpoint Failures**
   - Verify storage permissions
   - Check disk space
   - Review checkpoint frequency
   - Enable backup storage

## Advanced Features

### Custom Transformers
```lua
pipeline:add_transformer("custom", function(record)
    -- Custom transformation logic
    record.processed = true
    return record
end)
```

### Custom Metrics
```lua
pipeline:add_metric("custom_metric", function()
    return calculate_custom_value()
end)
```

### Hook Integration
```lua
pipeline:add_hook("pre_process", function(record)
    -- Pre-processing logic
end)

pipeline:add_hook("post_process", function(record)
    -- Post-processing logic
end)
```

## Testing

### Unit Tests
```bash
# Run unit tests
cargo test -p data-pipeline
```

### Integration Tests
```bash
# Run with test data
./scripts/test-pipeline.sh
```

### Load Testing
```bash
# Generate load
./scripts/load-test.sh --records 100000 --workers 10
```

## Contributing

See the main llmspell contribution guide. Key areas:
- Additional data source adapters
- Custom transformation functions
- Monitoring integrations
- Performance optimizations

## License

Same as llmspell project

## Support

- Documentation: See blueprint.md for architecture details
- Issues: Report via main llmspell repository
- Examples: Check cookbook/data-pipeline.lua for patterns
# RAG Configuration Migration Guide

This guide helps you transition RAG configurations from development to production environments.

## Migration Path Overview

```
Development → Testing → Staging → Production
```

Each stage has different requirements for performance, reliability, and resource usage.

## Stage 1: Development to Testing

### Starting Point: `rag-development.toml`

```toml
[rag.vector_storage]
backend = "mock"           # Change to real backend
dimensions = 384           # May need to increase
max_memory_mb = 512       # Increase for real data

[rag.embedding]
cache_enabled = false     # Enable for performance testing
batch_size = 4           # Increase for throughput
```

### Migration Steps

1. **Switch to Real Backend**
   ```toml
   backend = "hnsw"  # Use real vector storage
   ```

2. **Enable Persistence**
   ```toml
   persistence_path = "./test-data/rag"
   ```

3. **Increase Resources**
   ```toml
   max_memory_mb = 1024
   batch_size = 16
   ```

4. **Enable Basic Caching**
   ```toml
   [rag.cache]
   search_cache_enabled = true
   search_cache_size = 100
   ```

## Stage 2: Testing to Staging

### Target: Similar to `rag-basic.toml`

```toml
[rag]
enabled = true
multi_tenant = false  # Enable if needed

[rag.vector_storage]
dimensions = 768      # Production-grade embeddings
max_memory_mb = 2048  # Adequate resources
```

### Migration Checklist

- [ ] Increase vector dimensions for better accuracy
- [ ] Enable monitoring features
- [ ] Set up proper persistence paths
- [ ] Configure appropriate retry logic
- [ ] Test with production-like data volumes

### Key Changes

```toml
# From testing
[rag.embedding]
timeout_seconds = 10
max_retries = 1

# To staging
[rag.embedding]
timeout_seconds = 30
max_retries = 3
```

## Stage 3: Staging to Production

### Target: `rag-production.toml`

### Critical Production Features

1. **Enable Multi-Tenancy** (if required)
   ```toml
   [rag]
   multi_tenant = true
   
   [rag.multi_tenant_settings]
   strict_isolation = true
   max_vectors_per_tenant = 100000
   ```

2. **Add Monitoring**
   ```toml
   [rag.monitoring]
   enable_metrics = true
   enable_health_checks = true
   alert_on_errors = true
   error_threshold_per_minute = 10
   ```

3. **Configure Security**
   ```toml
   [rag.security]
   enable_audit_logging = true
   enable_rate_limiting = true
   rate_limit_per_user = 100
   enable_input_validation = true
   ```

4. **Set Up Backups**
   ```toml
   [rag.backup]
   enable_auto_backup = true
   backup_interval_hours = 24
   backup_retention_days = 30
   ```

## Performance Optimization Path

### From Basic to Performance

If you need maximum performance, migrate from `rag-basic.toml` to `rag-performance.toml`:

1. **Increase Vector Dimensions**
   ```toml
   # Basic
   dimensions = 384
   
   # Performance
   dimensions = 1536  # Higher accuracy
   ```

2. **Optimize HNSW Parameters**
   ```toml
   # Basic
   m = 16
   ef_construction = 200
   ef_search = 50
   
   # Performance
   m = 48
   ef_construction = 500
   ef_search = 200
   ```

3. **Maximize Caching**
   ```toml
   # Basic
   cache_size = 10000
   
   # Performance
   cache_size = 100000
   ```

## Migration Validation

### Pre-Migration Checklist

- [ ] Backup existing data
- [ ] Document current configuration
- [ ] Test new configuration in isolated environment
- [ ] Prepare rollback plan
- [ ] Schedule maintenance window (for production)

### Testing Script

```bash
#!/bin/bash
# Test migration from old to new config

OLD_CONFIG="rag-development.toml"
NEW_CONFIG="rag-production.toml"

# Test with old config
echo "Testing old configuration..."
LLMSPELL_CONFIG=$OLD_CONFIG ./validate-rag-configs.sh

# Test with new config
echo "Testing new configuration..."
LLMSPELL_CONFIG=$NEW_CONFIG ./validate-rag-configs.sh

# Run performance comparison
echo "Performance comparison..."
# Add your performance tests here
```

### Validation Points

1. **Functional Testing**
   - Document ingestion works
   - Search returns expected results
   - Multi-tenant isolation (if applicable)

2. **Performance Testing**
   - Query latency acceptable
   - Memory usage within limits
   - Throughput meets requirements

3. **Reliability Testing**
   - Retry logic functions
   - Error handling works
   - Backup/restore successful

## Common Migration Issues

### Issue 1: Memory Constraints

**Symptom**: Out of memory errors after migration

**Solution**:
```toml
# Reduce memory usage
max_memory_mb = 2048  # Lower limit
cache_size = 5000     # Smaller cache
batch_size = 16       # Smaller batches
```

### Issue 2: Slow Performance

**Symptom**: Queries taking too long

**Solution**:
```toml
# Optimize for speed
[rag.vector_storage.hnsw]
ef_search = 100       # Increase search quality
num_threads = 8       # More parallelism

[rag.cache]
search_cache_enabled = true
search_cache_size = 10000
```

### Issue 3: Provider Failures

**Symptom**: Embedding provider timeouts

**Solution**:
```toml
[rag.embedding]
timeout_seconds = 60  # Longer timeout
max_retries = 5      # More retries
batch_size = 8       # Smaller batches
```

## Rollback Procedure

If migration fails:

1. **Stop the application**
   ```bash
   systemctl stop llmspell
   ```

2. **Restore old configuration**
   ```bash
   cp /backup/old-config.toml /etc/llmspell/config.toml
   ```

3. **Restore data (if needed)**
   ```bash
   rm -rf /var/lib/llmspell/rag
   tar -xzf /backup/rag-backup.tar.gz -C /var/lib/llmspell/
   ```

4. **Restart with old config**
   ```bash
   systemctl start llmspell
   ```

## Environment-Specific Settings

### Development
- Use mock providers when possible
- Disable persistence
- Minimal caching
- Debug logging enabled

### CI/CD Pipeline
- Use deterministic test data
- Mock external dependencies
- Fast timeouts
- Disable retries

### Staging
- Mirror production settings
- Use separate API keys
- Enable monitoring
- Test with real data subset

### Production
- Full monitoring and alerting
- Backup and recovery enabled
- Security features active
- Optimized for your workload

## Monitoring Migration Success

### Key Metrics to Track

1. **Performance Metrics**
   - Query latency (p50, p95, p99)
   - Throughput (queries/second)
   - Cache hit rate

2. **Resource Metrics**
   - Memory usage
   - CPU utilization
   - Disk I/O

3. **Business Metrics**
   - User satisfaction
   - Error rate
   - Availability

### Sample Monitoring Query

```lua
-- Monitor RAG performance
local start = os.clock()
local results = RAG.search({ query = "test", top_k = 10 })
local elapsed = os.clock() - start

-- Log if slow
if elapsed > 0.1 then
    print(string.format("Slow RAG query: %.3fs", elapsed))
end
```

## Post-Migration Tasks

1. **Update Documentation**
   - Configuration changes
   - New features enabled
   - Performance characteristics

2. **Train Team**
   - New monitoring dashboards
   - Alert procedures
   - Troubleshooting guide

3. **Optimize Further**
   - Analyze actual usage patterns
   - Fine-tune cache sizes
   - Adjust resource limits

## Support and Resources

- [Configuration Reference](README.md)
- [Troubleshooting Guide](../../../docs/troubleshooting/rag.md)
- [Performance Tuning](../../../docs/performance/rag-tuning.md)
- [Community Forum](https://github.com/username/llmspell/discussions)
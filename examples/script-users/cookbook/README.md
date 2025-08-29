# LLMSpell Cookbook - Production Patterns

**Status**: 🚀 **Phase 8.10.6** - RAG patterns enhanced with multi-tenancy, sessions, and cost optimization

Enterprise-grade patterns for building production LLMSpell applications. Each pattern demonstrates battle-tested solutions to common production challenges.

## 📊 Pattern Collection

**11 Production Patterns**: 8 core patterns (v0.7.0) + 3 RAG patterns (v0.8.0)

## 📚 Pattern Categories

### 🛡️ Reliability & Resilience
- **[01 - Error Handling](error-handling.lua)** - Comprehensive error recovery strategies
- **[02 - Rate Limiting](rate-limiting.lua)** - API quota and throttling management

### ⚡ Performance & Optimization
- **[03 - Caching](caching.lua)** - High-performance caching strategies
- **[06 - Performance Monitoring](performance-monitoring.lua)** - Observability and metrics

### 🤝 Integration & Orchestration
- **[04 - Multi-Agent Coordination](multi-agent-coordination.lua)** - Agent collaboration patterns
- **[05 - Webhook Integration](webhook-integration.lua)** - External system integration

### 🔒 Security & State
- **[07 - Security Patterns](security-patterns.lua)** - Input validation and secure handling
- **[08 - State Management](state-management.lua)** - Versioning and persistence

### 🔍 RAG & Knowledge Management (Phase 8 - NEW)
- **[RAG-01 - Multi-Tenant RAG](rag-multi-tenant.lua)** - Isolated vector stores per tenant
- **[RAG-02 - Session RAG](rag-session.lua)** - Conversational memory with context
- **[RAG-03 - RAG Cost Optimization](rag-cost-optimization.lua)** - Efficient embedding strategies

## 🚀 Quick Start

### Basic Patterns (No API Key Required)
```bash
# Error handling patterns
./target/debug/llmspell run examples/script-users/cookbook/error-handling.lua

# Rate limiting strategies
./target/debug/llmspell run examples/script-users/cookbook/rate-limiting.lua

# Caching patterns
./target/debug/llmspell run examples/script-users/cookbook/caching.lua
```

### Agent Patterns (Requires API Key)
```bash
# Multi-agent coordination
./target/debug/llmspell -c examples/script-users/configs/example-providers.toml \
  run examples/script-users/cookbook/multi-agent-coordination.lua
```

### State Patterns (Optional Config)
```bash
# With persistence
./target/debug/llmspell -c examples/script-users/configs/state-enabled.toml \
  run examples/script-users/cookbook/state-management.lua

# In-memory only
./target/debug/llmspell run examples/script-users/cookbook/state-management.lua
```

### RAG Patterns (Phase 8 - Requires RAG Config)
```bash
# RAG-01: Multi-tenant RAG system
./target/debug/llmspell -c examples/script-users/configs/rag-production.toml \
  run examples/script-users/cookbook/rag-multi-tenant.lua

# RAG-02: Session-based RAG
./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml \
  run examples/script-users/cookbook/rag-session.lua

# RAG-03: Cost optimization patterns
./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml \
  run examples/script-users/cookbook/rag-cost-optimization.lua
```

## 📊 Pattern Overview

| Pattern | Version | Complexity | Prerequisites | Time | Key Features |
|---------|---------|------------|---------------|------|--------------|
| 01 - Error Handling | v0.7.0 | PRODUCTION | None | <3s | Safe invocation, retry, circuit breaker, aggregation |
| 02 - Rate Limiting | v0.7.0 | PRODUCTION | None | <2s | Token bucket, sliding window, adaptive limiting |
| 03 - Caching | v0.7.0 | PRODUCTION | None | <3s | LRU, TTL-based, write-through, statistics |
| 04 - Multi-Agent | v0.7.0 | PRODUCTION | API Key | <30s | Delegation, pipelines, consensus building |
| 05 - Webhooks | v0.7.0 | PRODUCTION | Network | <5s | Retry logic, signatures, batching, circuit breaker |
| 06 - Performance | v0.7.0 | PRODUCTION | None | <3s | Timing, memory tracking, percentiles, reports |
| 07 - Security | v0.7.0 | PRODUCTION | None | <2s | Validation, injection prevention, audit logging |
| 08 - State Mgmt | v0.7.0 | PRODUCTION | Optional | <3s | Versioning, migration, conflict resolution |
| RAG-01 - Multi-Tenant | v0.8.0 | PRODUCTION | RAG Config | <20s | Tenant isolation, quota management, admin ops |
| RAG-02 - Session RAG | v0.8.0 | INTERMEDIATE | RAG Config | <15s | Conversational memory, context windows, replay |
| RAG-03 - Cost Opt | v0.8.0 | INTERMEDIATE | RAG Config | <20s | Caching, batching, tiered processing, budgets |

## 🎯 When to Use Each Pattern

### Error Handling
Use when you need:
- Robust error recovery in production
- Graceful degradation under failures
- Comprehensive error reporting
- Retry strategies with backoff

### Rate Limiting
Use when you need:
- API quota management
- Prevention of resource abuse
- Fair resource allocation
- Traffic shaping and throttling

### Caching
Use when you need:
- Reduced API latency
- Lower operational costs
- Improved response times
- Reduced backend load

### Multi-Agent Coordination
Use when you need:
- Complex workflow orchestration
- Specialized agent collaboration
- Parallel processing with agents
- Consensus among multiple AI systems

### Webhook Integration
Use when you need:
- External system notifications
- Event-driven architectures
- Third-party integrations
- Asynchronous processing

### Performance Monitoring
Use when you need:
- Production observability
- Performance bottleneck detection
- SLA monitoring
- Resource utilization tracking

### Security Patterns
Use when you need:
- Input validation and sanitization
- Protection against injection attacks
- Secure credential handling
- Comprehensive audit trails

### State Management
Use when you need:
- Data persistence across restarts
- State versioning and history
- Distributed state handling
- Schema migration support

### Multi-Tenant RAG (RAG-01)
Use when you need:
- Isolated vector stores per customer
- Tenant-specific knowledge bases
- Usage tracking and quotas
- Cross-tenant admin operations
- Enterprise SaaS with strict data isolation

### Session RAG (RAG-02)
Use when you need:
- Conversational memory in chat apps
- Context-aware responses
- Session replay capabilities
- Temporary knowledge stores
- Research assistants with contextual memory

### RAG Cost Optimization (RAG-03)
Use when you need:
- Reduced embedding API costs (up to 70% savings)
- Efficient document processing
- Smart caching strategies
- Budget enforcement
- High-volume document ingestion

## 🆕 Phase 8 RAG Enhancements

The three RAG patterns (RAG-01, RAG-02, RAG-03) introduce production-ready vector storage patterns:

- **Multi-Tenancy**: Complete isolation between customer knowledge bases
- **Session Management**: Conversational memory with automatic cleanup
- **Cost Optimization**: Reduce embedding costs by up to 70% with smart caching
- **HNSW Algorithm**: High-performance approximate nearest neighbor search
- **Bi-temporal Metadata**: Track both event time and ingestion time
- **TTL Support**: Automatic document expiration for compliance

## 🏗️ Production Architecture

### Layered Defense Strategy
```
User Input → Validation → Rate Limiting → Caching → Processing → State → Response
     ↓           ↓            ↓            ↓          ↓         ↓         ↓
  Security    Security    Performance   Performance  Error   Persistence Monitoring
  Patterns    Patterns     Patterns      Patterns   Handling  Patterns   Patterns
```

### Multi-Agent Architecture
```
Coordinator Agent
    ├── Research Agent (parallel)
    ├── Analysis Agent (parallel)
    └── Review Agent (sequential)
         └── Webhook notification
```

## 📝 Best Practices

### General Guidelines
1. **Always validate input** - Never trust external data
2. **Implement rate limiting** - Protect against abuse
3. **Cache strategically** - Balance freshness vs performance
4. **Monitor everything** - You can't fix what you can't measure
5. **Handle errors gracefully** - Expect and plan for failures
6. **Version your state** - Enable rollback and migration
7. **Secure by default** - Apply defense in depth
8. **Document patterns** - Make them discoverable and reusable

### Performance Tips
- Use caching to reduce API calls
- Implement connection pooling
- Batch operations when possible
- Monitor memory usage patterns
- Set appropriate timeouts

### Security Tips
- Validate all user input
- Use parameterized queries
- Implement rate limiting
- Log security events
- Encrypt sensitive data
- Use secure credential storage

### Reliability Tips
- Implement circuit breakers
- Use exponential backoff
- Add health checks
- Plan for graceful degradation
- Test failure scenarios
- Monitor error rates

## 🔗 Learning Path

1. **Start Here**: [Getting Started](../getting-started/) - Learn basics (6 examples)
2. **Then Here**: [Features](../features/) - Explore capabilities
3. **You Are Here**: Cookbook - Production patterns (11 patterns including Phase 8 RAG)
4. **Next**: [Applications](../applications/) - Complete systems (9 applications)

## 📚 Additional Resources

- [LLMSpell API Reference](../../../docs/user-guide/api/lua/README.md)
- [Architecture Guide](../../../docs/technical/master-architecture-vision.md)
- [Configuration Guide](../configs/README.md)
- [Tool Catalog](../../../docs/user-guide/tools-catalog.md)

## 🤝 Contributing

To add a new cookbook pattern:
1. Ensure it solves a real production problem
2. Include comprehensive error handling
3. Add performance considerations
4. Document security implications
5. Provide clear usage examples
6. Test with various configurations

## 📄 License

These patterns are provided as examples for LLMSpell users. Feel free to adapt them for your production needs.
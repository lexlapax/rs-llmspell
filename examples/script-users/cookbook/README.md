# LLMSpell Cookbook - Production Patterns

Enterprise-grade patterns for building production LLMSpell applications. Each pattern demonstrates battle-tested solutions to common production challenges.

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

### 🔍 RAG & Knowledge Management
- **[09 - Multi-Tenant RAG](rag-multi-tenant.lua)** - Isolated vector stores per tenant
- **[10 - Session RAG](rag-session.lua)** - Conversational memory with context
- **[11 - RAG Cost Optimization](rag-cost-optimization.lua)** - Efficient embedding strategies

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

### RAG Patterns (Requires RAG Config)
```bash
# Multi-tenant RAG system
./target/debug/llmspell -c examples/script-users/configs/rag-production.toml \
  run examples/script-users/cookbook/rag-multi-tenant.lua

# Session-based RAG
./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml \
  run examples/script-users/cookbook/rag-session.lua

# Cost optimization patterns
./target/debug/llmspell -c examples/script-users/configs/rag-basic.toml \
  run examples/script-users/cookbook/rag-cost-optimization.lua
```

## 📊 Pattern Overview

| Pattern | Complexity | Prerequisites | Time | Key Features |
|---------|------------|---------------|------|--------------|
| Error Handling | PRODUCTION | None | <3s | Safe invocation, retry, circuit breaker, aggregation |
| Rate Limiting | PRODUCTION | None | <2s | Token bucket, sliding window, adaptive limiting |
| Caching | PRODUCTION | None | <3s | LRU, TTL-based, write-through, statistics |
| Multi-Agent | PRODUCTION | API Key | <30s | Delegation, pipelines, consensus building |
| Webhooks | PRODUCTION | Network | <5s | Retry logic, signatures, batching, circuit breaker |
| Performance | PRODUCTION | None | <3s | Timing, memory tracking, percentiles, reports |
| Security | PRODUCTION | None | <2s | Validation, injection prevention, audit logging |
| State Mgmt | PRODUCTION | Optional | <3s | Versioning, migration, conflict resolution |
| Multi-Tenant RAG | PRODUCTION | RAG Config | <20s | Tenant isolation, quota management, admin ops |
| Session RAG | INTERMEDIATE | RAG Config | <15s | Conversational memory, context windows, replay |
| RAG Cost Opt | INTERMEDIATE | RAG Config | <20s | Caching, batching, tiered processing, budgets |

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

### Multi-Tenant RAG
Use when you need:
- Isolated vector stores per customer
- Tenant-specific knowledge bases
- Usage tracking and quotas
- Cross-tenant admin operations

### Session RAG
Use when you need:
- Conversational memory in chat apps
- Context-aware responses
- Session replay capabilities
- Temporary knowledge stores

### RAG Cost Optimization
Use when you need:
- Reduced embedding API costs
- Efficient document processing
- Smart caching strategies
- Budget enforcement

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

1. **Start Here**: [Getting Started](../getting-started/) - Learn basics
2. **Then Here**: [Features](../features/) - Explore capabilities
3. **You Are Here**: Cookbook - Production patterns
4. **Next**: [Applications](../applications/) - Complete systems

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
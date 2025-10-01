# TODO.md Updates for Fleet-Based Architecture

## Summary of Changes

Replace complex runtime-per-session architecture with simple OS-level process isolation using fleet management. Each kernel runs one runtime, multiple clients share that runtime. Different requirements = different kernel processes.

## Phase 10 Task Updates

### KEEP AS-IS (No Changes Needed)

#### Tasks 10.1-10.6: ✅ ALL COMPLETE
- Unix Daemon Infrastructure
- Signal Handling
- Kernel Service
- Logging
- CLI Integration
- Jupyter Protocol
**Status**: These work perfectly for fleet model

#### Tasks 10.7-10.11: Debug Architecture
**Status**: NO CHANGES NEEDED
**Reason**: Each kernel process has isolated debug state
```bash
# Each kernel has its own ExecutionManager and DAPBridge
# Breakpoints naturally isolated by process boundaries
```

#### Task 10.12: Production Hardening ✅ COMPLETE
**Status**: Already complete, works for fleet

#### Task 10.13: REPL Service ✅ COMPLETE
**Status**: Works correctly - each kernel is a REPL service
```bash
# Multiple REPL kernels with isolation
llmspell kernel start --port 9555  # REPL 1
llmspell kernel start --port 9556  # REPL 2
```

#### Task 10.14: Example Application ✅ COMPLETE
**Status**: Keep existing, add fleet examples

#### Task 10.15: Integration Testing ✅ COMPLETE
**Status**: Keep existing tests, add fleet tests

#### Task 10.16: Documentation ✅ COMPLETE
**Status**: Keep existing, add fleet documentation

#### Task 10.17: Cleanup ✅ COMPLETE
**Status**: Already done

---

### REPLACE/REDEFINE

#### Task 10.18: ~~Client Registry~~ → Fleet Manager Implementation
**Old**: Complex in-kernel ClientRegistry with session mapping (5 hours)
**New**: Simple external fleet manager for process orchestration (4 hours)

**Priority**: CRITICAL
**Estimated Time**: 4 hours
**Description**: Implement fleet manager for multiple kernel processes
**Acceptance Criteria**:
- [x] Shell script implementation (llmspell-fleet)
- [x] Process lifecycle management (spawn/stop/list)
- [x] PID tracking and cleanup
- [x] Port allocation
- [x] Health checking
- [x] Connection file management

**Deliverables**:
1. `fleet/llmspell-fleet` - Bash implementation (DONE)
2. `fleet/fleet_manager.py` - Python implementation (DONE)
3. Documentation updates

---

#### Task 10.19: ~~Resource Limits~~ → Fleet Registry & Service Discovery
**Old**: Complex in-kernel resource tracking (14 hours)
**New**: Simple registry of running kernels (3 hours)

**Priority**: HIGH
**Estimated Time**: 3 hours
**Description**: Registry for kernel discovery and routing
**Acceptance Criteria**:
- [ ] JSON registry of running kernels
- [ ] Kernel capability metadata (language, config, resources)
- [ ] Client routing logic (find or spawn matching kernel)
- [ ] Dead kernel cleanup
- [ ] HTTP endpoint for discovery (optional)

**Implementation**:
```python
class FleetRegistry:
    def find_kernel(requirements) -> KernelInfo
    def register_kernel(kernel_info) -> None
    def cleanup_dead_kernels() -> None
```

---

#### Task 10.20: ~~Docker Complex~~ → Docker Fleet Orchestration
**Old**: Complex multi-runtime container (9 hours)
**New**: Simple docker-compose fleet (2 hours)

**Priority**: MEDIUM
**Estimated Time**: 2 hours
**Description**: Docker-based fleet orchestration
**Acceptance Criteria**:
- [x] docker-compose.yml for multi-kernel setup
- [x] Per-kernel resource limits (memory, CPU)
- [x] Health checks
- [x] Volume management for configs/logs
- [ ] Dockerfile updates (if needed)

**Deliverable**: `fleet/docker-compose.yml` (DONE)

---

#### Task 10.21: ~~Complex Metrics~~ → Fleet Monitoring
**Old**: Complex in-kernel metrics per runtime (15 hours)
**New**: Simple process-level monitoring (3 hours)

**Priority**: LOW
**Estimated Time**: 3 hours
**Description**: Basic monitoring for kernel fleet
**Acceptance Criteria**:
- [ ] Process metrics collection (memory, CPU, connections)
- [ ] Fleet-wide metrics aggregation
- [ ] Prometheus exporter (optional)
- [ ] Simple dashboard or status command

**Implementation**:
```python
def collect_fleet_metrics():
    for kernel in fleet.list_kernels():
        yield {
            "kernel_id": kernel.id,
            "memory_mb": get_process_memory(kernel.pid),
            "cpu_percent": get_process_cpu(kernel.pid),
            "uptime": get_process_uptime(kernel.pid)
        }
```

---

---

## Updated Phase 10 Timeline

### Original Timeline (Complex Architecture)
- Days 1-19: Core implementation ✅ COMPLETE
- Days 20-21: Client Registry (5 hours) → Runtime-per-session complexity
- Days 21-22: Resource Limits (14 hours) → Complex tracking
- Days 22-23: Docker (9 hours) → Multi-runtime container
- Days 23-24: Metrics (15 hours) → Complex per-runtime metrics
**Total**: 24 days + 43 hours of complex work

### New Timeline (Fleet Architecture)
- Days 1-19: Core implementation ✅ COMPLETE
- Day 20: Fleet Manager (4 hours) → Simple process management
- Day 20: Fleet Registry (3 hours) → Service discovery
- Day 21: Docker Fleet (2 hours) → Simple compose
- Day 21: Fleet Monitoring (3 hours) → Process metrics
- Day 22: Examples & Testing (3 hours)
- Day 22: Client Router (4 hours, optional)
**Total**: 22 days + 19 hours of simple work

### Savings
- **Time**: 24 hours saved (43 hours → 19 hours)
- **Complexity**: 0 kernel code changes vs ~3000 lines
- **Risk**: Low (external only) vs High (architecture change)
- **Testing**: Minimal vs Extensive

---

## Implementation Order

### Phase 1: Core Fleet (Day 20)
1. ✅ Create `fleet/llmspell-fleet` shell script
2. ✅ Create `fleet/fleet_manager.py` Python version
3. Test basic spawn/stop/list operations
4. Create fleet registry JSON structure

### Phase 2: Docker Support (Day 21 AM)
1. ✅ Create `fleet/docker-compose.yml`
2. Test Docker-based fleet
3. Document Docker workflow

### Phase 3: Monitoring (Day 21 PM)
1. Add metrics collection to fleet manager
2. Create simple metrics endpoint
3. Optional: Prometheus exporter

### Phase 4: Examples & Docs (Day 22)
1. Create fleet usage examples
2. Update documentation
3. Integration tests
4. Optional: Client router

---

## Migration Guide

### For Existing Users
```bash
# Old way (single kernel, would need runtime-per-session)
llmspell kernel start --daemon

# New way (fleet of simple kernels)
fleet spawn --config openai.toml    # Kernel 1
fleet spawn --config anthropic.toml # Kernel 2
fleet list                           # See all kernels
```

### For Developers
```python
# Old approach (would need complex ClientRegistry)
kernel = IntegratedKernel()
kernel.register_client(client_id)  # Complex session mapping

# New approach (simple process isolation)
kernel1 = fleet.spawn_kernel(config="openai.toml")
kernel2 = fleet.spawn_kernel(config="anthropic.toml")
# Each kernel is isolated by process boundary
```

### For Production
```yaml
# Docker Compose deployment
docker-compose -f fleet/docker-compose.yml up -d

# Kubernetes (future)
kubectl apply -f fleet/k8s/
```

---

## Benefits Summary

1. **No Kernel Changes**: Use existing code as-is
2. **Simple Architecture**: Unix process model
3. **True Isolation**: OS process boundaries
4. **Standard Tools**: ps, kill, docker, systemd
5. **Fast Implementation**: 19 hours vs 43 hours
6. **Low Risk**: External orchestration only
7. **Incremental**: Start simple, add features as needed

---

## Action Items

1. **Review and approve** this fleet-based approach
2. **Update TODO.md** with these changes
3. **Test fleet implementations** (shell and Python)
4. **Create fleet documentation**
5. **Add fleet examples**
6. **Optional**: Implement client router

The fleet approach delivers the same functionality with 1/6th the complexity and no kernel code changes.
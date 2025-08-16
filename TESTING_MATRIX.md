# Testing Matrix for Example Migration
Generated: 2025-08-16

## Summary Statistics
- **Total files**: 103 (89 Lua, 1 Rust, 6 Shell, 7 TOML)
- **Working examples**: ~15 files
- **Broken/Outdated**: ~70 files  
- **Test/Runner files**: ~10 files
- **Config files**: 7 files

## Testing Results by Directory

### examples/lua/ (4 files)
| File | Status | Category | Target Location |
|------|--------|----------|-----------------|
| `debug_globals.lua` | ✅ Works | No Dependencies | script-users/features/ |
| `run-all-examples.lua` | 🧪 Test Runner | Test File | tests-as-examples/runners/ |
| `run-integration-demos.lua` | 🧪 Test Runner | Test File | tests-as-examples/runners/ |
| `run-performance-benchmarks.lua` | 🧪 Test Runner | Test File | tests-as-examples/benchmarks/ |

### examples/lua/agents/ (10 files)
| File | Status | Category | Target Location |
|------|--------|----------|-----------------|
| `agent-api-comprehensive.lua` | ❌ Error | Broken/Outdated | script-users/features/ (after fix) |
| `agent-async-example.lua` | ✅ Works | No Dependencies | script-users/features/ |
| `agent-composition.lua` | ❌ Error | Broken/Outdated | script-users/cookbook/ (after fix) |
| `agent-coordinator.lua` | ❌ Error | Broken/Outdated | script-users/cookbook/ (after fix) |
| `agent-monitor.lua` | ❌ Error | Broken/Outdated | script-users/applications/ (after fix) |
| `agent-orchestrator.lua` | ❌ Error | Broken/Outdated | script-users/applications/ (after fix) |
| `agent-processor.lua` | ✅ Works | No Dependencies | script-users/features/ |
| `agent-simple-benchmark.lua` | ✅ Works | Test File | tests-as-examples/benchmarks/ |
| `agent-simple-demo.lua` | ✅ Works | No Dependencies | script-users/getting-started/ |
| `agent-simple.lua` | ❌ Error | Broken (deprecated API) | Remove (duplicate of agent-simple-demo) |

### examples/lua/tools/ (12 files)
| File | Status | Category | Target Location |
|------|--------|----------|-----------------|
| `tools-data.lua` | ❌ Error | Broken/Outdated | script-users/features/ (after fix) |
| `tools-filesystem.lua` | ✅ Works | No Dependencies | script-users/features/ |
| `tools-integration.lua` | ❌ Error | Broken/Outdated | script-users/cookbook/ (after fix) |
| `tools-media.lua` | ❌ Error | Broken/Outdated | script-users/features/ (after fix) |
| `tools-performance.lua` | ✅ Works | Test File | tests-as-examples/benchmarks/ |
| `tools-run-all.lua` | 🧪 Test Runner | Test File | tests-as-examples/runners/ |
| `tools-security.lua` | ❌ Error | Broken/Outdated | script-users/features/ (after fix) |
| `tools-showcase.lua` | ❌ Error | Broken/Outdated | script-users/getting-started/ (after fix) |
| `tools-system.lua` | ❌ Error | Broken/Outdated | script-users/features/ (after fix) |
| `tools-utility-reference.lua` | ❌ Error | Broken/Outdated | script-users/features/ (after fix) |
| `tools-utility.lua` | ✅ Works | No Dependencies | script-users/features/ |
| `tools-web.lua` | ❌ Error | Broken/Outdated | script-users/features/ (after fix) |
| `tools-workflow.lua` | ❌ Error | Broken/Outdated | script-users/cookbook/ (after fix) |

### examples/lua/workflows/ (9 files)
| File | Status | Category | Target Location |
|------|--------|----------|-----------------|
| `workflow-agent-integration.lua` | ❌ Error | API Keys Required | script-users/cookbook/ (after fix) |
| `workflow-basics-conditional.lua` | ❌ Error | Broken/Outdated | script-users/getting-started/ (after fix) |
| `workflow-basics-loop.lua` | ❌ Error | Broken/Outdated | script-users/getting-started/ (after fix) |
| `workflow-basics-parallel.lua` | ❌ Error | Broken/Outdated | script-users/getting-started/ (after fix) |
| `workflow-basics-sequential.lua` | ❌ Error | Broken/Outdated | script-users/getting-started/ (after fix) |
| `workflow-conditional.lua` | ❌ Error | Broken/Outdated | Remove (duplicate of basics) |
| `workflow-loop.lua` | ❌ Error | Broken/Outdated | Remove (duplicate of basics) |
| `workflow-parallel.lua` | ❌ Error | Broken/Outdated | Remove (duplicate of basics) |
| `workflow-sequential.lua` | ❌ Error | Broken/Outdated | Remove (duplicate of basics) |

### examples/state_persistence/ (5 files)
| File | Status | Category | Target Location |
|------|--------|----------|-----------------|
| `basic_operations.lua` | ✅ Works | No Dependencies | script-users/features/ |
| `basic_operations.rs` | 🦀 Rust | Rust Example | rust-developers/ |
| `run_quick_start.sh` | 🐚 Shell | Runner Script | Update and keep |
| `README.md` | 📝 Doc | Documentation | Keep with examples |
| `configs/basic.toml` | ⚙️ Config | Config File | script-users/configs/ |

### examples/configs/ (7 files)
All .toml configuration files → Move to `script-users/configs/`

### Shell Scripts (6 files)
| File | Status | Category | Action |
|------|--------|----------|--------|
| `run-all-agent-examples.sh` | 🐚 Shell | Runner | Update paths |
| `run-all-tools-examples.sh` | 🐚 Shell | Runner | Update paths |
| `run-workflow-examples.sh` | 🐚 Shell | Runner | Update paths |
| `run-all-lua-examples.sh` | 🐚 Shell | Runner | Update paths |
| `run-agent-examples.sh` | 🐚 Shell | Runner | Update paths |
| `run_quick_start.sh` | 🐚 Shell | Runner | Update paths |

## Migration Categories Summary

### Group A: No Dependencies (Ready to Migrate) - 8 files
- `debug_globals.lua`
- `agent-async-example.lua`
- `agent-processor.lua`
- `agent-simple-demo.lua`
- `tools-filesystem.lua`
- `tools-utility.lua`
- `basic_operations.lua`
- `agent-simple-benchmark.lua` (test file)

### Group B: Config Required - 7 files
- All .toml files in examples/configs/
- State persistence configs

### Group C: API Keys Required - ~10 files
- Most agent examples (when fixed)
- workflow-agent-integration.lua

### Group D: Test Files - ~15 files
- All run-*.lua files
- *-benchmark.lua files
- *-performance.lua files
- Test runners

### Group E: Broken/Outdated (Needs Fixing) - ~70 files
- Most workflow examples (API changes)
- Most tool examples (API changes)
- Several agent examples (deprecated Agent.create())

### Group F: Duplicates to Remove - ~5 files
- `agent-simple.lua` (keep agent-simple-demo.lua)
- `workflow-conditional.lua` (keep workflow-basics-conditional.lua)
- `workflow-loop.lua` (keep workflow-basics-loop.lua)
- `workflow-parallel.lua` (keep workflow-basics-parallel.lua)
- `workflow-sequential.lua` (keep workflow-basics-sequential.lua)

## Action Items

1. **Immediate Migration** (Group A):
   - Move 8 working files to appropriate locations
   - Add metadata headers during migration

2. **Fix Critical Examples**:
   - Update deprecated Agent.create() calls to Agent.builder()
   - Fix tool invocation APIs
   - Update workflow APIs

3. **Test File Organization**:
   - Move all test/benchmark files to tests-as-examples/
   - Update runner scripts for new structure

4. **Configuration**:
   - Consolidate all configs in script-users/configs/
   - Create example config for providers

5. **Remove Duplicates**:
   - Delete redundant workflow examples
   - Keep only the -basics- versions
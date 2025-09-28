# LLMSpell Scripts

> 🚀 **Comprehensive automation, testing, and deployment tools for the LLMSpell project**

**🔗 Navigation**: [← Project Root](../) | [Developer Guide](../docs/developer-guide/) | [Contributing](../CONTRIBUTING.md)

---

## 📁 Directory Structure

```
scripts/
├── 📊 quality/      # Code quality & CI/CD tools
├── 🧪 testing/      # Test execution & coverage
├── 🛠️  utilities/    # Helper scripts & tools
└── 🚢 fleet/        # Fleet orchestration & monitoring
```

## 🎯 Quick Navigation

| Category | Purpose | Key Scripts | Documentation |
|----------|---------|-------------|---------------|
| **[Quality](./quality/)** | Code standards & CI | `quality-check-*.sh`, `ci-test.sh` | [📖 Quality README](./quality/README.md) |
| **[Testing](./testing/)** | Test execution | `test-by-tag.sh`, `test-coverage.sh` | [📖 Testing README](./testing/README.md) |
| **[Utilities](./utilities/)** | Helper tools | `llmspell-easy.sh`, `backup_maintenance.sh` | [📖 Utilities README](./utilities/README.md) |
| **[Fleet](./fleet/)** | Fleet management | `llmspell-fleet`, `fleet_dashboard.py` | [📖 Fleet README](./fleet/README.md) |

## 🚀 Essential Scripts

### For Developers

```bash
# Quick quality check before commit
./quality/quality-check-fast.sh

# Run specific tests
./testing/test-by-tag.sh unit

# Find relevant examples
./utilities/find-examples.sh agent

# Start local kernel fleet
./fleet/llmspell-fleet start 3
```

### For DevOps

```bash
# Full CI validation
./quality/ci-test.sh full

# Generate coverage reports
./testing/test-coverage.sh all html

# Manage backups
./utilities/backup_maintenance.sh

# Monitor fleet health
./fleet/fleet_dashboard.py
```

### For End Users

```bash
# Easy launcher (zero config)
./utilities/llmspell-easy.sh

# Find examples by complexity
./utilities/find-examples.sh --tag beginner
```

## 📊 Quality Scripts (`quality/`)

**Purpose:** Maintain code quality, run CI/CD pipelines, validate applications

### Key Scripts
- **`quality-check-minimal.sh`** - Format & lint (~5s)
- **`quality-check-fast.sh`** - Add tests & docs (~1min)
- **`quality-check.sh`** - Full validation (~5min)
- **`ci-test.sh`** - CI pipeline runner
- **`validate_applications.py`** - Application validation

### Quick Usage
```bash
cd quality

# Pre-commit check
./quality-check-minimal.sh

# Pre-push validation
./quality-check-fast.sh

# Full PR validation
./quality-check.sh
```

**Application Test Matrix:**

| Layer | Agents | Applications | Runtime | Complexity |
|-------|--------|-------------|---------|------------|
| 1 - Universal | 2-3 | file-organizer, research-collector | <30s | Basic agents |
| 2 - Power User | 4 | content-creator | ~30s | Conditional workflows |
| 3 - Business | 5-7 | personal-assistant, communication-manager, code-review-assistant | 30-60s | State persistence |
| 4 - Professional | 8 | process-orchestrator, knowledge-base | 60-90s | Complex orchestration, RAG |
| 5 - Expert | 21 | webapp-creator | 8-10min | Full app generation |

**Current Status:** ✅ 9/9 applications passing (100% success rate)

[→ Full Quality Documentation](./quality/README.md)

## 🧪 Testing Scripts (`testing/`)

**Purpose:** Execute tests, analyze coverage, manage test suites

### Key Scripts
- **`test-by-tag.sh`** - Run tests by category
- **`test-coverage.sh`** - Generate coverage reports
- **`run-llmspell-tests.sh`** - Comprehensive runner
- **`list-tests-by-tag.sh`** - Discover test categories
- **`test-multiple-tags.sh`** - Batch test execution

### Available Test Tags
- `unit` - Library unit tests only
- `integration` - Integration tests only
- `tool` - Tests in llmspell-tools package
- `agent` - Tests in llmspell-agents package
- `workflow` - Workflow pattern tests
- `bridge` - Language bridge tests
- `fast` - Fast unit tests
- `slow` - Ignored tests marked as slow
- `external` - Tests requiring external services
- `all` - All tests including ignored

### Quick Usage
```bash
cd testing

# Run unit tests
./test-by-tag.sh unit

# Generate coverage
./test-coverage.sh all html

# Run full test suite
./run-llmspell-tests.sh comprehensive
```

[→ Full Testing Documentation](./testing/README.md)

## 🛠️ Utility Scripts (`utilities/`)

**Purpose:** Helper tools for users, developers, and administrators

### Key Scripts
- **`llmspell-easy.sh`** - User-friendly launcher
- **`find-examples.sh`** - Example discovery
- **`backup_maintenance.sh`** - Backup automation
- **`test_backup_integrity.sh`** - Backup validation

### Features
- Auto-detection of llmspell binary
- API key setup guidance
- Example discovery by tag/feature
- Automated backup management
- Integrity validation

### Quick Usage
```bash
cd utilities

# Launch LLMSpell easily
./llmspell-easy.sh

# Find examples
./find-examples.sh --feature workflow

# Create backup
./backup_maintenance.sh --backup-only
```

[→ Full Utilities Documentation](./utilities/README.md)

## 🚢 Fleet Scripts (`fleet/`)

**Purpose:** Kernel fleet management, monitoring, and orchestration

### Key Components
- **`llmspell-fleet`** - Fleet management CLI
- **`fleet_manager.py`** - Python fleet controller
- **`fleet_dashboard.py`** - Real-time monitoring
- **`log_aggregator.py`** - Centralized logging
- **Docker support** - Container orchestration
- **Prometheus metrics** - Monitoring integration

### Features
- **Multi-kernel orchestration** - Manage 1-100+ kernels
- **Resource monitoring** - CPU, memory, connections
- **Health checks** - Automatic recovery
- **Log aggregation** - Centralized analysis
- **Docker deployment** - Production-ready containers
- **Prometheus export** - Metrics integration

### Quick Usage
```bash
cd fleet

# Start fleet
./llmspell-fleet start 3

# Monitor fleet
python3 fleet_dashboard.py

# View aggregated logs
python3 log_aggregator.py tail

# Docker deployment
./docker-fleet.sh up
```

[→ Full Fleet Documentation](./fleet/README.md)

## 🔧 Common Workflows

### Development Workflow
```bash
# 1. Write code
# 2. Format and lint
scripts/quality/quality-check-minimal.sh

# 3. Run tests
scripts/testing/test-by-tag.sh unit

# 4. Full validation before PR
scripts/quality/quality-check.sh
```

### CI/CD Workflow
```bash
# Minimal checks (PR draft)
TEST_LEVEL=minimal scripts/quality/ci-test.sh

# Fast checks (PR ready)
TEST_LEVEL=fast scripts/quality/ci-test.sh

# Full validation (merge)
TEST_LEVEL=full scripts/quality/ci-test.sh

# Expensive tests (weekly)
RUN_EXPENSIVE_TESTS=1 scripts/quality/ci-test.sh
```

### Deployment Workflow
```bash
# 1. Create backup
scripts/utilities/backup_maintenance.sh

# 2. Deploy fleet
scripts/fleet/docker-fleet.sh up

# 3. Monitor health
scripts/fleet/fleet_dashboard.py

# 4. Check logs
scripts/fleet/log_aggregator.py monitor
```

## 📋 Best Practices

### Script Execution Order

**For New Contributors:**
1. `utilities/llmspell-easy.sh --setup`
2. `utilities/find-examples.sh --tag beginner`
3. `testing/test-by-tag.sh unit`

**For Regular Development:**
1. `quality/quality-check-minimal.sh` (pre-commit)
2. `testing/test-by-tag.sh <relevant-tag>` (during development)
3. `quality/quality-check-fast.sh` (pre-push)

**For Releases:**
1. `quality/quality-check.sh` (full validation)
2. `testing/test-coverage.sh all` (coverage analysis)
3. `quality/validate_applications.py` (application tests)

### Performance Targets

All scripts enforce these targets:
- **Tool initialization:** <10ms
- **State operations:** <5ms write, <1ms read
- **Test coverage:** >90% for units, >70% overall
- **Zero warnings:** Clean `cargo clippy`
- **Documentation:** >95% API coverage

## 🌍 Environment Variables

### Quality & Testing
- `SKIP_SLOW_TESTS=true` - Skip slow/external tests
- `RUN_EXPENSIVE_TESTS=1` - Enable webapp-creator test
- `TEST_LEVEL=<level>` - CI test level (minimal/fast/full)
- `REPORT_DIR=<path>` - Test report directory

### Fleet Management
- `FLEET_DIR=<path>` - Fleet data directory
- `MAX_KERNELS=<n>` - Maximum kernel limit
- `PROMETHEUS_PORT=<port>` - Metrics export port

### General
- `LLMSPELL_BIN=<path>` - Override llmspell binary
- `RUST_LOG=<level>` - Logging level
- `VERBOSE=true` - Verbose output

## 📈 CI/CD Integration

### GitHub Actions Workflows

**Main Test Workflow** (`.github/workflows/test.yml`)
- Runs on: Pull requests, pushes to main
- Cross-platform: Linux, macOS, Windows
- Performance benchmarking
- Security audit

**Scheduled Tests** (`.github/workflows/scheduled-tests.yml`)
- Daily: Full test suite (2 AM UTC)
- Weekly: Expensive tests (Sunday 3 AM UTC)
- Coverage analysis with thresholds
- Performance regression detection

### Test Reports

CI generates multiple report types:
- **HTML Report** - Visual test results
- **JSON Report** - Machine-readable data
- **Coverage Report** - Code coverage analysis
- **Performance Report** - Benchmark comparisons

All reports uploaded as GitHub Actions artifacts.

## 🔗 Related Documentation

- [Development Guide](../docs/development/README.md)
- [Testing Guide](../docs/development/testing.md)
- [CI/CD Pipeline](../.github/workflows/README.md)
- [Fleet Operations](../docs/operations/fleet.md)
- [Contributing Guidelines](../CONTRIBUTING.md)

## 🤝 Contributing

To contribute new scripts:

1. **Choose category:** quality, testing, utilities, or fleet
2. **Follow conventions:** Descriptive names with hyphens
3. **Add documentation:** Script header, usage examples
4. **Include error handling:** Proper exit codes, logging
5. **Update README:** Add to category README
6. **Test platforms:** macOS and Linux compatibility
7. **Submit PR:** With usage examples

See [CONTRIBUTING.md](../CONTRIBUTING.md) for detailed guidelines.

## 📊 Script Health Status

| Category | Scripts | Status | Last Updated |
|----------|---------|--------|--------------|
| Quality | 5 | ✅ Active | 2024-09-27 |
| Testing | 5 | ✅ Active | 2024-09-27 |
| Utilities | 4 | ✅ Active | 2024-09-27 |
| Fleet | 12+ | ✅ Active | 2024-09-27 |

---

**Need help?** Check the individual category READMEs or open an issue in the [project repository](https://github.com/yourusername/rs-llmspell/issues).
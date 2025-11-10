# Developer Setup Guide

**Complete development environment setup for rs-llmspell contributors**

**🔗 Quick Links**: [README](README.md) | [CONTRIBUTING](CONTRIBUTING.md) | [Developer Guide](docs/developer-guide/) | [Scripts](scripts/)

---

## Prerequisites

### Core Requirements

**Rust Toolchain** (mandatory)
```bash
# Install via rustup (https://rustup.rs)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version  # Should be >= 1.70
cargo --version

# Ensure stable toolchain with required components
rustup default stable
rustup component add rustfmt clippy
```

**Git** (mandatory)
```bash
# Verify installation
git --version  # Should be >= 2.30

# Configure for development
git config --global user.name "Your Name"
git config --global user.email "your.email@example.com"
```

**Docker** (recommended)
```bash
# Required for PostgreSQL testing and fleet management
# Install from https://docs.docker.com/get-docker/

# Verify installation
docker --version
docker compose version

# Test PostgreSQL setup
cd docker/postgres
docker compose up -d
docker exec llmspell_postgres_dev pg_isready -U llmspell
docker compose down
```

### Platform-Specific Setup

**macOS**
```bash
# Xcode Command Line Tools (provides essential build tools)
xcode-select --install

# Optional: Homebrew for additional tools
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Optional: Install PostgreSQL client tools
brew install postgresql@18

# Optional: Install Ollama for local LLM development
brew install ollama
```

**Linux (Debian/Ubuntu)**
```bash
# Build essentials (includes C/C++ compiler and libclang for RocksDB)
sudo apt-get update
sudo apt-get install -y build-essential libssl-dev pkg-config libclang-dev

# Testing utilities (used by CI)
sudo apt-get install -y bc jq

# Optional: PostgreSQL client tools
sudo apt-get install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh -y
sudo apt-get install -y postgresql-18 postgresql-server-dev-18

# Optional: Ollama for local LLM development
# See https://ollama.ai/download/linux
```

**Linux (Fedora/RHEL)**
```bash
# Build essentials (includes clang for RocksDB)
sudo dnf groupinstall "Development Tools"
sudo dnf install -y openssl-devel pkg-config clang-devel

# Testing utilities
sudo dnf install -y bc jq
```

**Linux (Arch)**
```bash
# Build essentials (includes clang for RocksDB)
sudo pacman -S base-devel clang llvm
```

**Linux (Alpine)**
```bash
# Build essentials (includes clang for RocksDB)
apk add build-base clang-dev llvm-dev
```

---

## Cargo Development Tools

### Essential Cargo Utilities

**cargo-tarpaulin** (code coverage - required for quality gates)
```bash
# Install
cargo install cargo-tarpaulin

# Verify
cargo tarpaulin --version

# Usage
./scripts/testing/test-coverage.sh all html
```

**cargo-audit** (security audit - required for CI)
```bash
# Install
cargo install cargo-audit

# Verify
cargo audit --version

# Usage
cargo audit  # Check for known vulnerabilities
```

### Optional Build Optimization

**sccache** (shared compilation cache - speeds up rebuilds)
```bash
# Install
cargo install sccache

# Configure environment
export RUSTC_WRAPPER=sccache
export SCCACHE_DIR=$HOME/.cache/sccache

# Verify cache is working
sccache --show-stats

# Add to shell profile for persistence
echo 'export RUSTC_WRAPPER=sccache' >> ~/.bashrc  # or ~/.zshrc
echo 'export SCCACHE_DIR=$HOME/.cache/sccache' >> ~/.bashrc
```

**Note on sccache**: Currently documented in [COMPILATION-PERFORMANCE.md](docs/archives/COMPILATION-PERFORMANCE.md) but not actively used in CI. Can provide 2-3x speedup for iterative development. Optional but recommended for active contributors.

### Build Performance & Memory Optimization

**⚠️ Important for systems with <16GB RAM**: This workspace has 20 crates with deep dependency trees (400+ transitive dependencies including surrealdb, aws-sdk-*, candle-*, arrow, parquet, rocksdb). Running `cargo test --workspace --all-features` can require **4-8GB RAM just for linking test binaries**. On limited-RAM systems, the linker may get OOM-killed by the kernel (signal 9 / "ld terminated with signal 9").

#### Understanding the Problem

**Dependency Graph Characteristics:**
- 20 workspace crates
- 400+ transitive dependencies with deep trees
- Heavy dependencies: surrealdb (100+ deps), aws-sdk-* (200+ deps), candle-* (50+ deps), arrow/parquet (80+ deps)
- Test dependencies add more weight: mockall, proptest, criterion, quickcheck

**Linker Memory Requirements:**
- Default linker must resolve ALL symbols for entire dependency graph in memory
- `--all-features` flag enables maximum dependency surface
- Single test binary: 4-8GB RAM for linking
- Workspace-wide tests: multiple binaries linking in parallel
- **Symptom**: "ld terminated with signal 9 [Killed]" or "collect2: fatal error"

**Current Optimizations (Always Active):**
- ✅ `incremental = true` - reuse previous compilation artifacts
- ✅ `codegen-units = 256` - parallelize codegen (reduces per-unit memory)
- ✅ `split-debuginfo = "unpacked"` - **NEW**: splits debug info into separate files (30-50% linker memory reduction)

#### Solution 1: Fast Linkers (Recommended)

**Best solution**: Modern linkers are 2-4x faster and use 60% less memory than default system linkers.

**Linux - mold linker:**
```bash
# Install via package manager (Debian/Ubuntu)
sudo apt install mold

# Or install via cargo
cargo install --locked mold

# Or build from source
git clone https://github.com/rui314/mold.git
cd mold && make -j$(nproc) && sudo make install

# Verify installation
mold --version
```

**macOS - lld linker (LLVM):**
```bash
# Install lld via Homebrew (separate package as of LLVM 21+)
brew install lld

# Verify installation
/opt/homebrew/opt/lld/bin/ld64.lld --version  # Apple Silicon
# OR
/usr/local/opt/lld/bin/ld64.lld --version     # Intel
```

**Activation:**
Edit `.cargo/config.toml` and uncomment the section for your platform:
- Linux x86_64: lines 47-49
- Linux ARM64: lines 53-55
- macOS Intel: lines 59-60
- macOS Apple Silicon: lines 64-65

**Verification:**
```bash
# After uncommenting, check linker is being used
cargo build 2>&1 | grep -E "(mold|ld64.lld)"

# Linux: Should see "mold" in linker invocation
# macOS: Should see "ld64.lld" in linker invocation
```

**Performance Impact:**
- Linking speed: 2-4x faster
- Memory usage: 60% reduction
- No code changes required
- Graceful fallback: If linker not found, cargo uses default linker

#### Solution 2: Per-Crate Testing

**Strategy**: Test individual crates instead of entire workspace to avoid linking everything simultaneously.

**Manual per-crate testing:**
```bash
# Test core functionality
cargo test -p llmspell-core --all-features
cargo test -p llmspell-tools --all-features
cargo test -p llmspell-agents --all-features

# Test memory system
cargo test -p llmspell-memory --all-features
cargo test -p llmspell-graph --all-features
cargo test -p llmspell-context --all-features

# Test infrastructure
cargo test -p llmspell-storage --all-features
cargo test -p llmspell-providers --all-features
```

**Automated per-crate testing (for CI/low-memory systems):**
```bash
# Test all crates sequentially
for crate in llmspell-{core,tools,agents,workflows,templates,memory,graph,context,rag,storage,providers,bridge,hooks,events,utils,config,security,tenancy,kernel,cli}; do
  echo "=== Testing $crate ==="
  cargo test -p $crate --all-features || exit 1
done
```

**Benefits:**
- Only links dependencies for one crate at a time
- 80% faster iteration when debugging specific component
- Uses 1/5th the memory of workspace-wide testing
- Better error isolation

#### Solution 3: System Resource Adjustments

**Linux - Add Swap Space (if you have sudo access):**
```bash
# Check current swap
free -h
swapon --show

# Add 16GB swap file
sudo fallocate -l 16G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Make persistent across reboots
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab

# Verify
free -h  # Should show 16G swap
```

**Reduce Parallel Build Jobs:**
```bash
# Limit concurrent compilation units (reduces peak memory)
cargo test --workspace --all-features -j 4  # Instead of default (# of CPUs)

# For very limited systems
cargo test --workspace --all-features -j 2
```

**Monitor Memory Usage:**
```bash
# Linux: Check for OOM kills in system logs
dmesg | grep -i "killed process"
dmesg | tail -50 | grep -i "out of memory"

# Real-time memory monitoring during build
watch -n 1 'free -h && ps aux | grep -E "(cargo|rustc|ld)" | head -20'
```

#### Solution 4: Reduced Feature Testing

**Strategy**: Test without enabling all features to reduce dependency surface.

**Common feature combinations:**
```bash
# Test with default features only (minimal dependencies)
cargo test --workspace

# Test with common development features
cargo test --workspace --features common

# Test specific feature sets
cargo test --workspace --features "sled,memory-only"
cargo test --workspace --features "postgres,hnsw"
```

**Trade-offs:**
- **Pro**: 50-70% reduction in linker memory requirements
- **Pro**: Faster iteration during development
- **Con**: May miss feature-specific bugs
- **Recommendation**: Use for local development, run `--all-features` in CI or before PR

**CI Strategy:**
```bash
# Local development: fast iteration
cargo test --workspace --features common

# Pre-commit: comprehensive validation
cargo test --workspace --all-features

# CI: full validation with better resources
cargo test --workspace --all-features --release
```

### Documentation Tools

**Node.js + npm** (for markdown link checking in CI - optional for local dev)
```bash
# Install Node.js from https://nodejs.org
# Or via package manager:
# macOS: brew install node
# Ubuntu: sudo apt-get install -y nodejs npm

# Install markdown-link-check (used by CI)
npm install -g markdown-link-check

# Usage (automated in CI)
markdown-link-check README.md
```

**lcov** (coverage report parsing - optional)
```bash
# macOS
brew install lcov

# Ubuntu/Debian
sudo apt-get install -y lcov

# Usage (automated by quality-check.sh)
lcov --summary lcov.info
```

---

## Quick Start

### Clone and Build

```bash
# Clone repository
git clone https://github.com/lexlapax/rs-llmspell.git
cd rs-llmspell

# Build with common features (recommended for development)
cargo build --features common
# Builds to: target/debug/llmspell (25MB)

# Alternative: Full build with all tools
cargo build --features full  # 35MB

# Alternative: Minimal build (core only)
cargo build  # 19MB

# Run CLI to verify
./target/debug/llmspell --version
```

### Run Quality Checks

```bash
# Minimal check (seconds) - format, clippy, compile
./scripts/quality/quality-check-minimal.sh

# Fast check (~1 min) - adds unit tests + docs
./scripts/quality/quality-check-fast.sh

# Full validation (5+ min) - required before PR
./scripts/quality/quality-check.sh
```

### Test Execution

```bash
# Run all tests via unified test runner
./scripts/testing/run-llmspell-tests.sh all

# Run specific test categories
./scripts/testing/run-llmspell-tests.sh unit
./scripts/testing/run-llmspell-tests.sh integration
./scripts/testing/run-llmspell-tests.sh fast  # unit + integration

# Run tests for specific components
./scripts/testing/test-by-tag.sh tool
./scripts/testing/test-by-tag.sh agent
./scripts/testing/test-by-tag.sh rag
./scripts/testing/test-by-tag.sh memory

# List all available test tags
./scripts/testing/list-tests-by-tag.sh all

# Generate coverage report
./scripts/testing/test-coverage.sh all html
# Opens tarpaulin-report.html
```

---

## Development Scripts Reference

### Scripts Structure

```
scripts/
├── quality/          Quality gates and CI/CD tools
│   ├── quality-check-minimal.sh   # <5s: format + clippy + compile
│   ├── quality-check-fast.sh      # ~1m: + unit tests + docs
│   ├── quality-check.sh           # 5m: full validation
│   └── ci-test.sh                 # CI-specific automation
├── testing/          Test execution and coverage
│   ├── run-llmspell-tests.sh      # Unified test runner (all categories)
│   ├── test-by-tag.sh             # Test by component tag
│   ├── list-tests-by-tag.sh       # List available test tags
│   ├── test-coverage.sh           # Coverage report generator
│   └── kernel-benchmark.sh        # Kernel performance benchmarks
├── utilities/        Helper scripts
│   ├── llmspell-easy.sh           # Zero-config launcher
│   ├── find-examples.sh           # Example discovery
│   └── backup_maintenance.sh      # State backup tools
└── fleet/            Docker fleet management
    ├── docker-fleet.sh            # Fleet orchestration
    └── test_fleet_*.sh            # Fleet testing scripts
```

### Quality Scripts

**quality-check-minimal.sh** - Fastest validation (<5s)
```bash
./scripts/quality/quality-check-minimal.sh

# Runs:
# 1. cargo fmt --all -- --check
# 2. cargo clippy --workspace --all-features --all-targets -- -D warnings
# 3. cargo check --workspace
# 4. Tracing pattern validation
```

**quality-check-fast.sh** - Development cycle (~1 min)
```bash
./scripts/quality/quality-check-fast.sh

# Runs minimal + unit tests + docs:
# 1-3. Same as minimal
# 4. cargo test --lib (core packages)
# 5. cargo doc --workspace --no-deps
```

**quality-check.sh** - Pre-commit/PR validation (5+ min)
```bash
./scripts/quality/quality-check.sh

# Comprehensive validation:
# 1. Code formatting
# 2. Clippy lints
# 3. Workspace build
# 4. Full test suite (with timeout 300s)
# 5. Performance benchmarks (optional, SKIP_BENCHMARKS=true)
# 6. Documentation build
# 7. Test coverage (requires cargo-tarpaulin)
# 8. Security audit (requires cargo-audit)

# Environment controls:
SKIP_SLOW_TESTS=true ./scripts/quality/quality-check.sh  # Skip slow/external tests
SKIP_BENCHMARKS=true ./scripts/quality/quality-check.sh  # Skip benchmarks
```

### Testing Scripts

**run-llmspell-tests.sh** - Unified test interface
```bash
# Run all tests
./scripts/testing/run-llmspell-tests.sh all

# Fast test suite (unit + integration)
./scripts/testing/run-llmspell-tests.sh fast

# Comprehensive suite (excludes external/benchmark)
./scripts/testing/run-llmspell-tests.sh comprehensive

# Specific test types
./scripts/testing/run-llmspell-tests.sh unit
./scripts/testing/run-llmspell-tests.sh integration
./scripts/testing/run-llmspell-tests.sh external
./scripts/testing/run-llmspell-tests.sh benchmark

# Component categories
./scripts/testing/run-llmspell-tests.sh tool
./scripts/testing/run-llmspell-tests.sh agent
./scripts/testing/run-llmspell-tests.sh workflow
./scripts/testing/run-llmspell-tests.sh bridge
./scripts/testing/run-llmspell-tests.sh memory

# With options
./scripts/testing/run-llmspell-tests.sh unit --verbose
./scripts/testing/run-llmspell-tests.sh integration --release

# Help
./scripts/testing/run-llmspell-tests.sh --help
```

**test-coverage.sh** - Generate coverage reports
```bash
# HTML report (default)
./scripts/testing/test-coverage.sh all html
open tarpaulin-report.html

# LCOV format (for CI)
./scripts/testing/test-coverage.sh all lcov

# JSON format
./scripts/testing/test-coverage.sh all json

# Coverage for specific test types
./scripts/testing/test-coverage.sh unit html
./scripts/testing/test-coverage.sh integration lcov
```

### Utility Scripts

**llmspell-easy.sh** - Zero-configuration launcher
```bash
# Interactive launcher for non-technical users
./scripts/utilities/llmspell-easy.sh

# Auto-detects:
# - llmspell binary location
# - API key configuration
# - Profile setup
```

**find-examples.sh** - Example discovery
```bash
./scripts/utilities/find-examples.sh

# Locates all examples across:
# - examples/script-users/
# - examples/rust-developers/
# - examples/templates/
```

### Fleet Scripts

**docker-fleet.sh** - Docker orchestration
```bash
# Build Docker image
./scripts/fleet/docker-fleet.sh build

# Start fleet with default profile
./scripts/fleet/docker-fleet.sh up

# Start with specific profile
./scripts/fleet/docker-fleet.sh up dev
./scripts/fleet/docker-fleet.sh up javascript

# Scale service
./scripts/fleet/docker-fleet.sh scale kernel-lua-openai 3

# View logs
./scripts/fleet/docker-fleet.sh logs
./scripts/fleet/docker-fleet.sh logs kernel-lua-openai

# Health check
./scripts/fleet/docker-fleet.sh health

# Stop and cleanup
./scripts/fleet/docker-fleet.sh down
./scripts/fleet/docker-fleet.sh clean
```

---

## Cargo Aliases

Configured in [.cargo/config.toml](.cargo/config.toml):

```bash
# Test execution
cargo test-all         # Test entire workspace
cargo test-unit        # Unit tests only
cargo test-integration # Integration tests only
cargo test-agent       # Agent tests
cargo test-scenario    # Scenario tests
cargo test-lua         # Lua bridge tests

# Benchmarks
cargo bench-all        # Run all benchmarks

# Quality checks
cargo fmt-check        # Check formatting without modifying
cargo clippy-all       # Clippy with warnings as errors
cargo doc-all          # Generate all docs

# Build variants
cargo build-all        # Build workspace with all features
cargo build-release    # Release build with all features
```

---

## Environment Variables

### Development Configuration

```bash
# Enable debug logging
export RUST_LOG=debug
export RUST_BACKTRACE=1

# Cargo build optimization
export CARGO_INCREMENTAL=1
export CARGO_TERM_COLOR=always

# Optional: sccache
export RUSTC_WRAPPER=sccache
export SCCACHE_DIR=$HOME/.cache/sccache

# Test configuration
export SKIP_SLOW_TESTS=true      # Skip slow tests in quality-check.sh
export SKIP_BENCHMARKS=true      # Skip benchmarks in quality-check.sh
```

### LLM Provider API Keys (for testing)

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."

# PostgreSQL (for storage testing)
export LLMSPELL_POSTGRES_URL="postgresql://llmspell:llmspell@localhost:5435/llmspell"

# Optional: Store in .env file (gitignored)
cat > .env << 'EOF'
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
LLMSPELL_POSTGRES_URL=postgresql://llmspell:llmspell@localhost:5435/llmspell
EOF
```

---

## Development Workflow

### Typical Development Cycle

```bash
# 1. Create feature branch
git checkout -b feature/your-feature

# 2. Make changes + write tests
$EDITOR src/...

# 3. Run minimal checks frequently (seconds)
./scripts/quality/quality-check-minimal.sh

# 4. Run fast checks before committing (~1 min)
./scripts/quality/quality-check-fast.sh

# 5. Commit with conventional format
git add .
git commit -m "feat: Add your feature description"

# 6. Run full validation before PR (5+ min)
./scripts/quality/quality-check.sh

# 7. Push and create PR
git push origin feature/your-feature
```

### Performance Development

```bash
# Benchmark specific component
cargo bench -p llmspell-tools
cargo bench -p llmspell-memory
cargo bench -p llmspell-storage

# Kernel performance benchmarks
./scripts/testing/kernel-benchmark.sh

# Profile with flamegraph (requires cargo-flamegraph)
cargo install flamegraph
cargo flamegraph --bench <benchmark_name>
```

### Database Development

```bash
# Start PostgreSQL for testing
cd docker/postgres
docker compose up -d

# Verify connection
docker exec llmspell_postgres_dev pg_isready -U llmspell

# Run database-specific tests
cargo test -p llmspell-storage --features postgres

# View logs
docker compose logs -f

# Stop PostgreSQL
docker compose down
```

---

## IDE Setup

### VS Code

Recommended extensions:
- **rust-analyzer** - Rust language support
- **CodeLLDB** - Debugging support
- **Even Better TOML** - TOML syntax highlighting
- **Error Lens** - Inline error display

Workspace settings (.vscode/settings.json):
```json
{
  "rust-analyzer.cargo.features": ["common"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.checkOnSave.extraArgs": ["--all-targets", "--all-features"],
  "[rust]": {
    "editor.formatOnSave": true,
    "editor.defaultFormatter": "rust-lang.rust-analyzer"
  }
}
```

### IntelliJ IDEA / CLion

1. Install Rust plugin
2. Configure Toolchain: Settings → Languages & Frameworks → Rust
3. Set Cargo command: `build --features common`
4. Enable Clippy: Settings → Rust → External Linters → Clippy

---

## Troubleshooting

### Common Issues

**Build failures with feature flags**
```bash
# Error: "features not found"
# Solution: Always specify features for builds
cargo build --features common    # Not: cargo build
```

**Slow compilation times**
```bash
# Solution 1: Use sccache (see above)
export RUSTC_WRAPPER=sccache

# Solution 2: Clean target directory periodically
cargo clean

# Solution 3: Use minimal features for development
cargo build  # 19MB, fastest
```

**Test failures with "connection refused"**
```bash
# PostgreSQL tests failing
# Solution: Start PostgreSQL container
cd docker/postgres && docker compose up -d
```

**Coverage generation fails**
```bash
# Error: "cargo-tarpaulin not found"
# Solution: Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Error: "tarpaulin timeout"
# Solution: Increase timeout or run specific coverage
./scripts/testing/test-coverage.sh unit html  # Faster than 'all'
```

**Clippy errors on CI but not locally**
```bash
# CI uses stricter flags
# Solution: Run exact CI command
cargo clippy --workspace --all-features --all-targets -- -D warnings
```

### Performance Issues

**Large target/ directory (>100GB)**
```bash
# Check size
du -sh target/

# Clean old artifacts
cargo clean

# Clean specific profiles
rm -rf target/debug/deps/*.rlib

# See: docs/archives/COMPILATION-PERFORMANCE.md for detailed analysis
```

**Memory usage during tests**
```bash
# Run tests sequentially to reduce memory pressure
cargo test -- --test-threads=1

# Run specific test categories
./scripts/testing/run-llmspell-tests.sh fast  # Skips external/benchmark
```

**Linker OOM (Out of Memory) Kills**

**Symptoms:**
```
error: linking with `cc` failed: exit status: 1
= note: collect2: fatal error: ld terminated with signal 9 [Killed]
        compilation terminated.
```

**Diagnosis:**
```bash
# Linux: Check system logs for OOM killer activity
dmesg | tail -50 | grep -i -E "(killed|out of memory|oom)"

# Check available memory
free -h  # Linux
vm_stat  # macOS

# Identify which process was killed
journalctl -xe | grep -i oom  # systemd systems
```

**Quick Fix (Immediate):**
```bash
# Test per-crate instead of entire workspace
cargo test -p llmspell-core --all-features
cargo test -p llmspell-memory --all-features
# etc... (see "Solution 2: Per-Crate Testing" above)
```

**Permanent Fix (Choose one):**
1. **Best**: Install fast linker (mold/lld) - see "Build Performance & Memory Optimization" section
2. **Linux**: Add swap space - see "Solution 3: System Resource Adjustments"
3. **Alternative**: Reduce parallelism - `cargo test -j 2`
4. **Development**: Use `--features common` instead of `--all-features`

**Root Cause:**
This workspace has 400+ dependencies with deep trees. With `--all-features`, test binaries require 4-8GB RAM just for linking. The default system linker loads the entire symbol table into memory. Modern linkers (mold/lld) use 60% less memory and are 2-4x faster.

---

## CI/CD Integration

### GitHub Actions Workflow

Configured in [.github/workflows/ci.yml](.github/workflows/ci.yml):

**Jobs:**
1. **quality** - Format, clippy, docs (ubuntu-latest)
2. **test** - Build + test matrix (ubuntu-latest, macos-latest)
3. **coverage** - Code coverage >90% (ubuntu-latest)
4. **security** - Security audit (ubuntu-latest)
5. **benchmarks** - Performance benchmarks (informational)
6. **quality-gates** - Final validation gate
7. **docs** - Documentation build + deployment

**Local CI simulation:**
```bash
# Run exact CI quality checks
./scripts/quality/ci-test.sh

# Run tests with same configuration as CI
cargo test --workspace --all-features
```

---

## Additional Resources

### Documentation
- [Developer Guide](docs/developer-guide/README.md) - 8 comprehensive guides
- [API Reference](docs/developer-guide/reference/) - Rust crate documentation
- [Scripts README](scripts/README.md) - Complete scripts documentation
- [CONTRIBUTING](CONTRIBUTING.md) - Contribution guidelines

### Learning Paths
- **Tool Developer**: 01-getting-started → 03-extending-components (Part 1)
- **RAG Developer**: 01-getting-started → 03-extending-components (Part 5)
- **Production Engineer**: 01-getting-started → 05-production-deployment
- **Bridge Developer**: 01-getting-started → 04-bridge-patterns

### Examples
- [Rust Developers](examples/rust-developers/) - 6 patterns
- [Script Users](examples/script-users/) - 60+ Lua examples
- [Templates](examples/templates/) - 10 workflow templates

### Performance Targets
| Component | Target | Measurement |
|-----------|--------|-------------|
| Tool init | <10ms | `cargo bench -p llmspell-tools` |
| Agent creation | <50ms | `cargo bench -p llmspell-agents` |
| Hook overhead | <2% | Performance tests |
| Vector search | <8ms @ 100K | `cargo bench -p llmspell-storage` |
| Memory ops | <2ms | `cargo bench -p llmspell-memory` |
| Template init | <2ms | `cargo bench -p llmspell-templates` |

---

## Getting Help

- **Questions**: [GitHub Discussions](https://github.com/lexlapax/rs-llmspell/discussions)
- **Bugs**: [GitHub Issues](https://github.com/lexlapax/rs-llmspell/issues)
- **Security**: Report privately via GitHub Security
- **Chat**: Check README for community channels

---

**Ready to contribute? Start with [CONTRIBUTING.md](CONTRIBUTING.md)!**

# Binary Size Analysis Report
**Date**: 2025-01-28
**Goal**: Identify unused dependencies and opportunities to reduce binary size

## Current Binary Size
- **Release binary**: 33.6MB (39MB debug)
- **.text section**: 23.0MB

## Top Binary Contributors (cargo bloat)
```
1. std           - 3.0MiB (13.0%) [unavoidable]
2. llmspell_bridge - 1.6MiB (7.0%) [our code]
3. llmspell_tools  - 1.4MiB (6.3%) [our code]
4. mlua          - 1.2MiB (5.3%) [Lua runtime - needed]
5. arrow_cast    - 1.2MiB (5.0%) [❌ SUSPICIOUS - why Apache Arrow?]
6. llmspell_kernel - 1.0MiB (4.4%) [our code]
7. arrow_array   - 674KB (2.9%) [❌ Apache Arrow]
8. chumsky       - 571KB (2.4%) [❌ Parser combinator - check usage]
9. h2            - 492KB (2.1%) [HTTP/2 - from reqwest?]
10. arrow_select - 450KB (1.9%) [❌ Apache Arrow]
11. parquet      - 440KB (1.9%) [❌ Parquet files - why?]
12. tera         - 436KB (1.8%) [❌ Template engine - check usage]
13. jsonschema   - 329KB (1.3%) [JSON schema validation]
14. pdf_extract  - 312KB (1.3%) [PDF parsing]
15. zstd_sys     - 304KB (1.3%) [compression]
16. brotli       - 292KB (1.2%) [compression]
17. sled         - 291KB (1.2%) [embedded database]
18. zmq_sys      - 266KB (1.1%) [ZeroMQ for Jupyter]
```

## Major Issues Identified

### 1. Apache Arrow & Parquet (2.8MB total!)
- arrow_cast (1.2MB) + arrow_array (674KB) + arrow_select (450KB) + parquet (440KB)
- **Total: ~2.8MB** - This is HUGE for features we likely don't use
- Need to find which crate pulls this in

### 2. Multiple Compression Libraries (900KB+)
- zstd_sys (304KB)
- brotli (292KB)
- lz4 (in kernel)
- flate2/gzip (in kernel)
- We use all 4 in state/backup/compression.rs - do we need all?

### 3. Template/Parser Libraries
- tera (436KB) - template engine
- chumsky (571KB) - parser combinators
- Need to verify if these are actually used

### 4. Duplicate Vector Libraries
- hnsw_rs in llmspell-kernel
- hnsw crate in llmspell-rag (unused!)
- Both doing the same thing

## Detailed Crate Analysis

### llmspell-tools (Major Offender)
**Problem**: Pulling in 5.5MB+ of dependencies
- **Apache Arrow + Parquet (2.8MB)**: Only used in csv_analyzer.rs for CSV->Parquet conversion
- **tera (436KB)**: Template engine, only used in template_engine.rs
- **jaq (571KB via chumsky)**: JSON query processor
- **pdf-extract (312KB)**: PDF parsing
- **xlsxwriter + calamine**: Excel file handling

### llmspell-kernel
**Issues**:
- **4 compression libraries**: zstd (304KB), brotli (292KB), lz4, flate2
  - All used in state/backup/compression.rs - do we need all 4 formats?
- **sled (291KB)**: Embedded database - check if actually needed
- **sysinfo (large)**: System monitoring - only used in 3 places
- **zmq (266KB)**: Required for Jupyter protocol

### llmspell-cli
**Issues**:
- **serde_yaml (deprecated)**: Used for YAML output format
- **tabled (large)**: Only used once in kernel.rs for table formatting
- **dialoguer**: Interactive prompts - barely used
- **indicatif**: Progress bars - used in one file
- **colored**: Terminal colors - used 3 times

### llmspell-bridge
**Issues**:
- **mlua (1.2MB)**: Lua runtime - REQUIRED, can't remove
- Pulls in all other crates' dependencies

### Dependency Duplication
- Multiple UUID versions: 1.7, 1.8, 1.11, 1.17
- Multiple chrono imports (11 different ways)
- Both hnsw and hnsw_rs crates

## Recommendations for Binary Size Reduction

### CRITICAL - Must Fix (Save ~3.5MB)

1. **Make Apache Arrow/Parquet Optional in llmspell-tools (Save 2.8MB)**
   - Create feature flag: `csv-parquet` for Arrow/Parquet support
   - Move csv_analyzer.rs behind feature flag
   - Default features should NOT include this
   - Impact: Removes arrow_cast, arrow_array, arrow_select, parquet
   ```toml
   [features]
   default = []
   csv-parquet = ["arrow", "parquet"]
   ```

2. **Remove unused hnsw crate from llmspell-rag (Save ~100KB)**
   - Currently using hnsw_rs in kernel, hnsw in rag
   - RAG doesn't actually use its hnsw dependency (0 imports found)
   - Just remove `hnsw = "0.11"` from llmspell-rag/Cargo.toml

3. **Make Template Engine Optional (Save 436KB)**
   - Put tera behind feature flag in llmspell-tools
   - Only used in template_engine.rs
   ```toml
   [features]
   templates = ["tera"]
   ```

4. **Make PDF Support Optional (Save 312KB)**
   - Put pdf-extract behind feature flag
   - Only used for PDF parsing in tools
   ```toml
   [features]
   pdf = ["pdf-extract"]
   ```

### MEDIUM Priority (Save ~1MB)

5. **Reduce Compression Libraries (Save ~600KB)**
   - Keep only gzip (flate2) and zstd
   - Remove brotli and lz4
   - Gzip is standard, zstd is best modern compression
   - Update CompressionType enum to remove unused variants

6. **Replace Heavy CLI Dependencies (Save ~300KB)**
   - Replace `tabled` with simple custom table formatting (10-20 lines)
   - Replace `dialoguer` with simple stdin reading (5-10 lines)
   - Replace `indicatif` with simple progress printing (10 lines)
   - Replace `colored` with ANSI codes directly (5 lines per color)

7. **Standardize Dependencies**
   - Use workspace-level UUID version (1.17 everywhere)
   - Use workspace-level chrono (0.4 everywhere)
   - Remove serde_yaml (deprecated), use serde_json for YAML-like output

### LOW Priority (Save ~200KB)

8. **Make Excel Support Optional**
   - Put xlsxwriter and calamine behind feature flag
   - Only needed for Excel file operations

9. **Make JSON Query Optional**
   - Put jaq-* crates behind feature flag (saves chumsky too)
   - Only used for advanced JSON querying

10. **Optimize sysinfo Usage**
    - Only 3 usage points - could replace with simple /proc reading on Linux
    - Or make it optional behind "monitoring" feature flag

## Feature Flag Strategy

Proposed feature structure:
```toml
# llmspell-tools/Cargo.toml
[features]
default = ["templates", "pdf"]  # Keep common features
full = ["csv-parquet", "excel", "json-query", "templates", "pdf"]
csv-parquet = ["arrow", "parquet"]
excel = ["xlsxwriter", "calamine"]
json-query = ["jaq-core", "jaq-interpret", "jaq-parse"]
templates = ["tera"]
pdf = ["pdf-extract"]

# llmspell-cli/Cargo.toml
[dependencies]
llmspell-tools = { path = "../llmspell-tools", default-features = false, features = ["templates", "pdf"] }
```

## Expected Savings

- **Immediate (no feature loss)**: ~3.5MB
  - Remove Arrow/Parquet: 2.8MB
  - Remove unused hnsw: 100KB
  - CLI dependency replacements: 300KB
  - Remove duplicate compression: 300KB

- **With feature flags (advanced features optional)**: ~5MB total
  - All of the above plus:
  - Optional template engine: 436KB
  - Optional PDF: 312KB
  - Optional Excel: 200KB+
  - Optional JSON query: 571KB

**Target: Reduce from 33.6MB to ~28MB (16% reduction) without losing core features**

## Quick Wins (Can do immediately)

1. Remove `hnsw = "0.11"` from llmspell-rag/Cargo.toml
2. Replace tabled usage with simple formatting in llmspell-cli/src/commands/kernel.rs
3. Standardize UUID versions in workspace Cargo.toml
4. Remove unused imports and dependencies

## Code to Replace Heavy Dependencies

### Replace tabled (in kernel.rs)
```rust
// Instead of using tabled, simple table formatting:
fn print_simple_table(headers: &[&str], rows: Vec<Vec<String>>) {
    let widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for (i, header) in headers.iter().enumerate() {
        print!("{:width$} ", header, width = widths[i]);
    }
    println!();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            print!("{:width$} ", cell, width = widths[i]);
        }
        println!();
    }
}
```

### Replace colored
```rust
// Simple ANSI colors
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const RESET: &str = "\x1b[0m";
fn red(s: &str) -> String { format!("{RED}{s}{RESET}") }
fn green(s: &str) -> String { format!("{GREEN}{s}{RESET}") }
```

### Replace indicatif progress bar
```rust
// Simple progress indicator
fn show_progress(current: usize, total: usize) {
    let percent = (current * 100) / total;
    print!("\rProgress: [{}{}] {}%",
           "=".repeat(percent / 5),
           " ".repeat(20 - percent / 5),
           percent);
}
```


## Action Plan for Implementation

### Phase 1: Quick Wins (1 hour, save ~500KB)
1. [ ] Remove `hnsw = "0.11"` from llmspell-rag/Cargo.toml
2. [ ] Replace tabled in kernel.rs with simple formatting
3. [ ] Standardize UUID to 1.17 across workspace
4. [ ] Replace colored with ANSI codes (3 usage points)

### Phase 2: Feature Flags (2 hours, save 2.8MB)
1. [ ] Add feature flags to llmspell-tools/Cargo.toml
2. [ ] Gate csv_analyzer.rs behind csv-parquet feature
3. [ ] Gate template_engine.rs behind templates feature
4. [ ] Gate pdf tools behind pdf feature
5. [ ] Update llmspell-cli to use minimal features

### Phase 3: Dependency Reduction (2 hours, save 600KB)
1. [ ] Remove brotli and lz4 compression support
2. [ ] Replace dialoguer with simple stdin
3. [ ] Replace indicatif with simple progress
4. [ ] Consider replacing serde_yaml

### Verification
1. [ ] Run `cargo bloat --release --crates -n 30` before/after
2. [ ] Ensure all tests pass with minimal features
3. [ ] Test with `--all-features` flag for full functionality
4. [ ] Document feature flags in README

## Final Summary

- **Current binary**: 33.6MB
- **Expected after Phase 1**: ~33MB
- **Expected after Phase 2**: ~30MB  
- **Expected after Phase 3**: ~28MB
- **Total reduction**: 5.6MB (16.7%)

The biggest win is making Apache Arrow/Parquet optional (2.8MB savings). This is a huge dependency for a feature that's only used in CSV analysis. Most users won't need CSV->Parquet conversion, so this should be opt-in rather than default.

Without removing any core functionality, we can reduce the binary by 5-6MB through better dependency management and feature flags.

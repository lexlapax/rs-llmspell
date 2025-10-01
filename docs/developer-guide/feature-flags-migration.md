# Feature Flags Migration Guide

## Overview
Starting with Phase 10.17.5, rs-llmspell has moved to a feature flag-based build system to reduce binary size and allow users to include only the dependencies they need. This guide helps existing users migrate to the new system.

## Breaking Changes
**Default build no longer includes heavy dependencies.** The following tools are now optional:
- Template engines (Tera, Handlebars)
- PDF processing
- CSV/Parquet analysis
- Excel support
- Archive handling
- Email capabilities
- Database connectivity

## Migration Steps

### 1. Assess Your Tool Usage
Run your existing scripts and note which tools they use:
```bash
# Check tool availability
./target/release/llmspell run your-script.lua
```

If you see errors like "Tool 'template_engine' not found", you need to enable the corresponding feature.

### 2. Choose Your Build Configuration

#### Option A: Common Build (Recommended)
If you use templates or PDF processing:
```bash
cargo build --release --features common
```

#### Option B: Full Build
If you need all tools (similar to old default):
```bash
cargo build --release --features full
```

#### Option C: Custom Build
Select only what you need:
```bash
# Example: Just templates and archives
cargo build --release --features templates,archives
```

### 3. Update Your Build Scripts
Replace:
```bash
cargo build --release
```

With your chosen configuration:
```bash
cargo build --release --features common  # or your selection
```

### 4. Runtime Tool Discovery
The good news: **No code changes required!** Tool discovery is automatic:
- `Tool.list()` only shows available tools
- Scripts gracefully handle missing optional tools
- Runtime errors clearly indicate which feature to enable

## Feature Mapping

| If you use...                | Enable feature    |
|------------------------------|------------------|
| `TemplateEngineTool`         | `templates`      |
| `PdfProcessorTool`           | `pdf`            |
| `CsvAnalyzerTool`            | `csv-parquet`    |
| `ExcelHandlerTool`           | `excel`          |
| `ArchiveHandlerTool`         | `archives`       |
| `JsonQueryTool`              | `json-query`     |
| `EmailTool` (SMTP)           | `email`          |
| `EmailTool` (AWS SES)        | `email-aws`      |
| Database operations          | `database`       |

## Size Comparison

| Build Type | Binary Size | Reduction from Old Default |
|------------|-------------|----------------------------|
| Minimal    | ~19MB       | -14.6MB (-43%)            |
| Common     | ~25MB       | -8.6MB (-26%)             |
| Full       | ~35MB       | +1.4MB (+4%)              |
| Old Default| ~33.6MB     | baseline                  |

## CI/CD Updates

Update your CI pipelines:

```yaml
# GitHub Actions example
- name: Build minimal
  run: cargo build --release --bin llmspell

- name: Build with common tools
  run: cargo build --release --features common

# Test all configurations
- name: Test minimal
  run: cargo test --no-default-features --features lua

- name: Test full
  run: cargo test --all-features
```

## Docker Updates

Update your Dockerfiles:

```dockerfile
# Minimal image
FROM rust:1.76 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin llmspell

# Common tools image
FROM rust:1.76 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features common --bin llmspell
```

## Troubleshooting

### Q: How do I know which features I need?
A: Run your application with minimal build. Missing tools will error with the tool name. Map the tool name to features using the table above.

### Q: Can I change features without rebuilding?
A: No, features are compile-time. You must rebuild to add/remove features.

### Q: Will my scripts break?
A: No, as long as you include the features for tools you use. The API hasn't changed.

### Q: What's the recommended configuration?
A: For most users, `common` provides a good balance. Use `minimal` for containers/embedded, `full` for development.

## Performance Impact

Feature flags have **zero runtime overhead**. The only differences are:
- Binary size (19-35MB depending on features)
- Compilation time (minimal builds faster)
- Tool availability at runtime

## Support

If you encounter issues migrating:
1. Check tool errors indicate which feature to enable
2. Consult the feature mapping table above
3. File an issue with your specific use case

## Future Compatibility

This feature system is designed for long-term stability:
- New tools will be added as optional features
- Core functionality remains in the minimal build
- Feature names are stable and won't change
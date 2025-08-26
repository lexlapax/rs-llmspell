# Contributing to rs-llmspell

Thank you for your interest in contributing to rs-llmspell! This guide will help you get started.

**🔗 Navigation**: [← Project Home](README.md) | [Developer Guide](docs/developer-guide/) | [Documentation](docs/)

---

## ⚡ Quick Start

For comprehensive development setup and workflows, see the **[Developer Guide](docs/developer-guide/README.md)**.

### Quick Commands Reference

```bash
# Before committing (fast - seconds)
./scripts/quality-check-minimal.sh

# Before PR (comprehensive - 5+ minutes)  
./scripts/quality-check.sh

# Run specific tests
cargo test -p llmspell-tools
cargo test -p llmspell-agents

# Check specific crate
cargo check -p llmspell-bridge
cargo clippy -p llmspell-core
```

## Development Workflow

1. **Fork and Clone**
   ```bash
   git clone https://github.com/yourusername/rs-llmspell
   cd rs-llmspell
   ```

2. **Create Feature Branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Run Quality Checks**
   ```bash
   # Quick checks before committing
   ./scripts/quality-check-minimal.sh
   
   # Full validation before PR
   ./scripts/quality-check.sh
   ```

4. **Make Changes**
   - Write code following existing patterns
   - Add tests for new functionality
   - Update documentation as needed

5. **Submit Pull Request**
   - Clear description of changes
   - Reference any related issues
   - Ensure all checks pass

## Quality Standards

- **Zero Warnings**: All code must compile without warnings (`cargo clippy -- -D warnings`)
- **Test Coverage**: Maintain 90%+ test coverage
- **Documentation**: Update docs for any API changes
- **Performance**: No regressions in benchmarks

## Where to Contribute

### Tools Development
- See [Tool Development Guide](docs/developer-guide/tool-development-guide.md)
- Tools must implement proper security levels
- Include comprehensive tests

### Documentation
- Keep documentation accurate and current
- Update examples when APIs change
- Fix any outdated information

### Bug Fixes
- Include tests that reproduce the issue
- Reference issue number in commit message
- Verify fix doesn't break existing functionality

### Performance
- Run benchmarks before and after changes
- Document any significant improvements
- Consider memory usage impacts

## Testing

```bash
# Run specific test categories
./scripts/test-by-tag.sh unit
./scripts/test-by-tag.sh integration
./scripts/test-by-tag.sh tool

# List available test tags
./scripts/list-tests-by-tag.sh all
```

## Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Use descriptive variable names
- Keep functions focused and small
- Comment complex logic

## Commit Messages

Use clear, descriptive commit messages:
- `feat: Add new web scraping tool`
- `fix: Resolve path traversal in file operations`
- `docs: Update agent API examples`
- `test: Add integration tests for workflows`

## Review Process

1. All PRs require at least one review
2. CI must pass (tests, clippy, formatting)
3. Documentation must be updated
4. Breaking changes need clear justification

## Security

- Never commit secrets or API keys
- Follow security guidelines in [Security Guide](docs/developer-guide/security-guide.md)
- Report security issues privately via GitHub Security

## Questions?

- Check [Developer Guide](docs/developer-guide/) for detailed information
- Open a [Discussion](https://github.com/lexlapax/rs-llmspell/discussions) for design questions
- File an [Issue](https://github.com/lexlapax/rs-llmspell/issues) for bugs

## Code of Conduct

- Be respectful and constructive
- Welcome newcomers and help them get started
- Focus on what is best for the community
- Show empathy towards other community members

Thank you for contributing to rs-llmspell!
# Quality Gates Configuration

This document outlines the quality gates that must be configured for the rs-llmspell repository to ensure code quality and stability.

## Branch Protection Rules

Configure the following branch protection rules in GitHub repository settings:

### Main Branch Protection
- **Branch name pattern**: `main`
- **Restrict pushes that create files**: ✅ Enabled
- **Require status checks to pass before merging**: ✅ Enabled
- **Require up-to-date branches**: ✅ Enabled
- **Require conversation resolution before merging**: ✅ Enabled
- **Require signed commits**: ✅ Enabled (recommended)
- **Require linear history**: ✅ Enabled (recommended)

### Required Status Checks
The following CI jobs must pass before any merge to main:

1. **Quality Checks**
   - `quality / Quality Checks`
   - Enforces: formatting, clippy lints, documentation builds

2. **Test Suite** 
   - `test / Test Suite`
   - Enforces: all unit/integration tests pass

3. **Code Coverage**
   - `coverage / Code Coverage`
   - Enforces: >90% test coverage threshold

4. **Security Audit**
   - `security / Security Audit`
   - Enforces: no known security vulnerabilities

5. **Documentation**
   - `docs / Documentation`
   - Enforces: documentation builds, link validation

## Quality Standards

### 1. Test Coverage (>90%)
- **Enforcement**: CI fails if coverage drops below 90%
- **Tool**: `cargo-tarpaulin` with JSON output parsing
- **Scope**: Workspace-wide line coverage
- **Reporting**: Codecov integration for tracking trends

### 2. Zero Compilation Warnings
- **Enforcement**: All CI jobs use `-D warnings` flag
- **Scope**: All workspace crates with all features
- **Tools**: 
  - `cargo build --workspace --all-features`
  - `cargo doc --workspace` with `RUSTDOCFLAGS="-D warnings"`

### 3. Clippy Lints (Deny Level)
- **Enforcement**: `cargo clippy --workspace --all-features -- -D warnings`
- **Rules**: Comprehensive ruleset including:
  - All default lints at deny level
  - Performance lints
  - Correctness lints
  - Style consistency
  - Complexity warnings

### 4. Code Formatting
- **Enforcement**: `cargo fmt --all -- --check`
- **Tool**: `rustfmt` with project-wide consistency
- **Scope**: All Rust code in workspace

### 5. Documentation Coverage (>95%)
- **Enforcement**: CI tracks documentation coverage percentage
- **Requirements**:
  - All public APIs documented with examples
  - All crate-level documentation complete
  - All README files with valid links
- **Tools**: 
  - Custom coverage calculation from `cargo doc` output
  - `cargo-deadlinks` for link validation
  - `markdown-link-check` for README validation

## Security Requirements

### 1. Dependency Auditing
- **Tool**: `cargo-audit` via GitHub Actions
- **Frequency**: Every CI run + weekly scheduled scans
- **Policy**: No known security vulnerabilities allowed

### 2. Dependency Updates
- **Tool**: Dependabot configuration
- **Frequency**: Weekly dependency update PRs
- **Scope**: Cargo dependencies + GitHub Actions

## Performance Baselines

### 1. Benchmark Regression Prevention
- **Tool**: `criterion` benchmarks
- **Policy**: Informational only (no blocking)
- **Purpose**: Track performance trends over time

## Repository Configuration Commands

### Setting up Branch Protection (Repository Admin)

```bash
# Using GitHub CLI (gh)
gh api repos/lexlapax/rs-llmspell/branches/main/protection \
  --method PUT \
  --field required_status_checks='{"strict":true,"contexts":["quality / Quality Checks","test / Test Suite","coverage / Code Coverage","security / Security Audit","docs / Documentation"]}' \
  --field enforce_admins=true \
  --field required_pull_request_reviews='{"required_approving_review_count":1,"dismiss_stale_reviews":true,"require_code_owner_reviews":true}' \
  --field restrictions=null \
  --field allow_force_pushes=false \
  --field allow_deletions=false
```

### Enabling Required Status Checks

1. Go to repository Settings → Branches
2. Click "Add rule" for main branch
3. Enable "Require status checks to pass before merging"
4. Select all required CI jobs from the list
5. Enable "Require branches to be up to date before merging"

## Local Development Commands

Developers should run these commands locally before pushing:

```bash
# Quality checks (same as CI)
cargo fmt --all                              # Format code
cargo clippy --workspace --all-features -- -D warnings  # Lint code
cargo build --workspace --all-features       # Build all
cargo test --workspace                       # Run tests
cargo doc --workspace --no-deps             # Build docs

# Optional: Run coverage locally
cargo tarpaulin --workspace --out Html       # Generate coverage report
```

## Violation Handling

### When Quality Gates Fail

1. **Test Coverage Below 90%**
   - Add missing tests for uncovered code paths
   - Review if any code can be removed or simplified
   - Update coverage threshold only with team approval

2. **Clippy Warnings**
   - Fix all clippy suggestions
   - Use `#[allow(clippy::specific_lint)]` only with justification
   - Document any intentional violations

3. **Documentation Coverage Below 95%**
   - Add missing rustdoc comments
   - Include examples in documentation
   - Verify all links are valid

4. **Security Vulnerabilities**
   - Update vulnerable dependencies immediately
   - If no update available, assess risk and document mitigation
   - Consider alternative dependencies

## Emergency Override Process

In rare emergency situations, quality gates may be bypassed:

1. **Approval Required**: Team lead + 1 additional team member
2. **Documentation**: Create issue documenting the override reason
3. **Follow-up**: Quality violation must be fixed in next PR
4. **Review**: Process reviewed in next team retrospective

## Quality Metrics Dashboard

Track these metrics over time:

- Test coverage percentage (target: >90%)
- Documentation coverage percentage (target: >95%)
- CI success rate (target: >95%)
- Average time to merge PRs
- Number of quality gate violations per month
- Security vulnerabilities discovered and time to resolution

## Configuration Files

- `.github/workflows/ci.yml` - Complete CI/CD pipeline
- `.github/dependabot.yml` - Automated dependency updates
- `.markdown-link-check.json` - Link validation configuration
- `Cargo.toml` - Workspace and quality tool configuration
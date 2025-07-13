# Task 3.1.4: External Tool Dependencies Documentation

## Overview

This document outlines the external dependencies added for the communication tools in Phase 3.1, including feature flags and configuration options.

## Dependencies Added

### Email Support

#### 1. **lettre** (v0.11)
- **Purpose**: SMTP email sending with async support
- **Feature Flag**: `email`
- **Features Used**:
  - `smtp-transport`: SMTP protocol support
  - `tokio1-rustls-tls`: Async TLS support with Tokio
  - `hostname`: Hostname resolution
  - `builder`: Message builder API

#### 2. **aws-sdk-ses** (v1.61)
- **Purpose**: AWS Simple Email Service integration
- **Feature Flag**: `email-aws`
- **Dependencies**: Requires `aws-config` for configuration

### Database Support

#### 1. **sqlx** (v0.8)
- **Purpose**: Async SQL database operations
- **Feature Flag**: `database`
- **Features Used**:
  - `runtime-tokio-rustls`: Tokio async runtime with Rustls TLS
  - `postgres`: PostgreSQL driver
  - `mysql`: MySQL driver
  - `sqlite`: SQLite driver
  - `chrono`: DateTime support
  - `uuid`: UUID support

## Feature Flags

### Tool-Level Features (llmspell-tools)

```toml
[features]
default = []
email = ["lettre"]
email-aws = ["aws-sdk-ses", "aws-config"]
database = ["sqlx"]
database-postgres = ["database", "sqlx/postgres"]
database-mysql = ["database", "sqlx/mysql"]
database-sqlite = ["database", "sqlx/sqlite"]
full = ["email", "email-aws", "database"]
```

### Feature Combinations

1. **No features** (default):
   - All communication tools return mock responses
   - No external dependencies required
   - Suitable for testing and development

2. **Email only** (`--features email`):
   - Enables SMTP email sending via lettre
   - SendGrid still uses mock (HTTP-based)

3. **Email with AWS** (`--features email,email-aws`):
   - Enables both SMTP and AWS SES
   - Requires AWS credentials

4. **Database only** (`--features database`):
   - Enables all database drivers (PostgreSQL, MySQL, SQLite)
   - Can be further restricted with specific database features

5. **Full features** (`--features full`):
   - Enables all email and database providers
   - Maximum functionality

## Configuration

### Email Configuration

#### Environment Variables

```bash
# SMTP Configuration
EMAIL_SMTP_HOST=smtp.gmail.com
EMAIL_SMTP_PORT=587
EMAIL_SMTP_USERNAME=user@example.com
EMAIL_SMTP_PASSWORD=app-specific-password

# SendGrid Configuration
EMAIL_SENDGRID_API_KEY=SG.xxxxx

# AWS SES Configuration
EMAIL_SES_ACCESS_KEY_ID=AKIA...
EMAIL_SES_SECRET_ACCESS_KEY=xxxxx
EMAIL_SES_REGION=us-east-1

# Default sender
EMAIL_DEFAULT_SENDER=noreply@example.com
```

### Database Configuration

#### Environment Variables

```bash
# PostgreSQL
DATABASE_POSTGRESQL_URL=postgres://user:pass@localhost/dbname

# MySQL
DATABASE_MYSQL_URL=mysql://user:pass@localhost/dbname

# SQLite
DATABASE_SQLITE_PATH=/path/to/database.db

# Security settings
DATABASE_ALLOW_DDL=false
DATABASE_ALLOW_DML=true
DATABASE_MAX_ROWS=1000
```

## Build Configuration

### CI/CD Pipeline

The `.github/workflows/rust.yml` file tests multiple feature combinations:

1. No features (default)
2. Email only
3. Database only
4. Email + Database
5. Full features

### Building with Features

```bash
# Build with email support only
cargo build -p llmspell-tools --features email

# Build with database support only
cargo build -p llmspell-tools --features database

# Build with all features
cargo build -p llmspell-tools --features full

# Build specific database support
cargo build -p llmspell-tools --features database-postgres
```

## Security Considerations

### Credential Management

1. **Environment Variables**: All credentials should be provided via environment variables
2. **No Hardcoding**: Never hardcode credentials in source code
3. **Secure Storage**: Use secure credential storage in production (e.g., AWS Secrets Manager)

### Feature Security

1. **Minimal Features**: Only enable features you need
2. **Database Security**: DDL operations disabled by default
3. **SQL Injection**: Built-in protection in DatabaseConnectorTool
4. **Rate Limiting**: Consider implementing rate limits for email sending

## Implementation Details

### Conditional Compilation

Both EmailSenderTool and DatabaseConnectorTool use conditional compilation:

```rust
#[cfg(feature = "email")]
{
    // Real SMTP implementation
}

#[cfg(not(feature = "email"))]
{
    // Mock implementation
}
```

This ensures:
1. Zero overhead when features are disabled
2. Clear separation between real and mock implementations
3. Compile-time feature validation

### Error Handling

All external operations properly handle errors:
- Network failures
- Authentication errors
- Invalid configurations
- Timeout handling

## Testing

### Unit Tests

Tests work with both enabled and disabled features:
- Mock implementations for feature-disabled builds
- Integration tests for feature-enabled builds

### Integration Testing

```bash
# Test without features
cargo test -p llmspell-tools

# Test with email features
cargo test -p llmspell-tools --features email

# Test with all features
cargo test -p llmspell-tools --features full
```

## Performance Impact

### Binary Size

- Base (no features): Baseline size
- With email: +~2MB (lettre and dependencies)
- With database: +~5MB (sqlx and drivers)
- Full features: +~7MB total increase

### Compilation Time

- Additional dependencies increase initial compilation time
- Subsequent builds benefit from cargo's incremental compilation
- CI/CD uses dependency caching to minimize impact

## Future Considerations

1. **Additional Providers**:
   - Mailgun email provider
   - Redis database support
   - MongoDB support

2. **Feature Granularity**:
   - Separate features per email provider
   - Optional compression for database connections

3. **Performance Optimizations**:
   - Connection pooling improvements
   - Async operation batching

## Conclusion

The external dependencies have been successfully integrated with:
- ✅ Feature flags for optional functionality
- ✅ Proper error handling for all scenarios
- ✅ CI/CD pipeline configuration
- ✅ Security best practices
- ✅ Comprehensive documentation

This implementation allows users to choose their desired level of functionality while maintaining a clean separation between mock and real implementations.
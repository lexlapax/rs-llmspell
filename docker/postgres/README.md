# PostgreSQL Development Environment (Phase 13b.2)

PostgreSQL 18 with VectorChord 0.5.3 extension for llmspell experimental storage backends.

## Quick Start

```bash
# Start PostgreSQL container
cd docker/postgres
docker compose up -d

# Check container status
docker compose ps

# View logs
docker compose logs -f postgres

# Stop container
docker compose down

# Stop and remove data volume (DESTRUCTIVE)
docker compose down -v
```

## Configuration

**Image**: `ghcr.io/tensorchord/vchord-postgres:pg18-v0.5.3`
- PostgreSQL 18.0
- VectorChord 0.5.3 (with pgvector 0.8.1 dependency)

**Database**: `llmspell_dev`
**User**: `llmspell`
**Password**: `llmspell_dev_pass`
**Port**: `5432`

**Connection String**:
```
postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev
```

## Extensions

Extensions are initialized via `init-scripts/01-extensions.sql` (populated in Task 13b.2.3):
- `vchord` (VectorChord 0.5.3 - HNSW vector search)
- `vector` (pgvector 0.8.1 - installed as CASCADE dependency)
- `pgcrypto` (cryptographic functions)
- `uuid-ossp` (UUID generation)

## Health Checks

Container includes automatic health checks:
- **Command**: `pg_isready -U llmspell`
- **Interval**: 10s
- **Timeout**: 5s
- **Retries**: 5

Check health:
```bash
docker compose ps  # Look for "healthy" status
```

## Performance Settings

```
shared_preload_libraries = 'vchord'
max_connections = 100
shared_buffers = 512MB
```

## Data Persistence

PostgreSQL data stored in Docker volume `postgres_data` (survives container restarts).

Remove volume to reset database:
```bash
docker compose down -v
```

## Troubleshooting

**Container won't start**:
```bash
# Check logs
docker compose logs postgres

# Verify image downloaded
docker images | grep vchord

# Remove and recreate
docker compose down -v
docker compose up -d
```

**Port 5432 already in use**:
```bash
# Check what's using port 5432
sudo lsof -i :5432

# Stop local PostgreSQL if running
sudo systemctl stop postgresql
```

**Extensions not loading**:
```bash
# Connect to database
docker compose exec postgres psql -U llmspell -d llmspell_dev

# List extensions
\dx

# Check vchord loaded
SELECT * FROM pg_extension WHERE extname = 'vchord';
```

## Development Workflow

1. Start container: `docker compose up -d`
2. Run tests: `cargo test --features postgres`
3. Stop when done: `docker compose down`

## CI/CD Integration

CI strategy (designed in Task 13b.2.0.5):
- Docker Compose used in GitHub Actions (Linux only)
- macOS tests skip PostgreSQL (Docker not available)
- Target: <10min total CI runtime (<7.5min Linux with PostgreSQL)

## Notes

- **VectorChord requires CASCADE**: Init scripts MUST use `CREATE EXTENSION vchord CASCADE;` (not standalone)
- **Development only**: Password is hardcoded for dev convenience (DO NOT use in production)
- **Phase 13b.2 scope**: Infrastructure only (no storage operations until Phase 13b.4+)

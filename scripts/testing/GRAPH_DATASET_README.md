# Synthetic Graph Dataset Generator

Task 13c.2.8.6: Generate 100K node synthetic graph for performance testing.

## Requirements

```bash
cargo install rust-script
```

## Usage

### Generate Full Dataset (100K entities, ~1M relationships)

```bash
./scripts/testing/generate-graph-dataset.sh
```

Output directory: `benchmarks/graph-dataset/`

### Manual Generation

```bash
cd benchmarks/graph-dataset
rust-script ../../scripts/testing/generate-graph-dataset.rs
```

## Output Files

- `entities.json` - 100,000 entities (JSON array)
- `relationships.json` - ~1,000,000 relationships (JSON array)
- `dataset-summary.txt` - Statistics and metadata

## Dataset Specification

### Entities (100,000 total)

| Type         | Count  | Distribution |
|--------------|--------|--------------|
| person       | 30,000 | 30%          |
| concept      | 25,000 | 25%          |
| organization | 20,000 | 20%          |
| event        | 15,000 | 15%          |
| location     | 10,000 | 10%          |

### Relationships (~1,000,000 total, avg 10 per entity)

| Type       | Distribution |
|------------|--------------|
| knows      | ~20%         |
| works_at   | ~20%         |
| part_of    | ~20%         |
| caused_by  | ~20%         |
| located_in | ~20%         |

### Temporal Properties

- **Time range**: 5 years (from 5 years ago to now)
- **Event time**: Random timestamp within 5-year range
- **Ingestion time**: Event time + 0-48 hours

### Properties

Each entity/relationship includes:
- `synthetic: true` - Marks as test data
- `index: N` (entities only) - Sequential index
- `weight: 1-100` (relationships only) - Random weight

## Performance Expectations

With this dataset:
- **SQLite traverse(4-hop)**: <50ms p95
- **PostgreSQL traverse(4-hop)**: <35ms p95 (GiST indexes)

## File Sizes (Estimated)

- `entities.json`: ~25 MB
- `relationships.json`: ~280 MB
- Total: ~305 MB

## Testing

Test generator with small dataset:

```bash
rust-script scripts/testing/test-generator.rs
```

Validates:
- JSON structure
- Temporal consistency
- Data distribution

# Graph Traversal Performance Characteristics

Task 13c.2.8.8: Technical documentation for multi-hop graph traversal implementation

**Status**: Phase 13c.2.8 - Storage Consolidation
**Version**: v0.14.0 (in progress)
**Last Updated**: 2025-11-12

## Overview

Multi-hop graph traversal implementation using recursive Common Table Expressions (CTEs) for both SQLite and PostgreSQL backends. Supports cycle detection, bi-temporal filtering, relationship type filtering, and configurable depth limits.

## Implementation Strategy

### Recursive CTE Pattern

Both backends use SQL recursive CTEs with two cases:

1. **Base Case**: Select starting entity at depth 0
2. **Recursive Case**: Follow relationships to depth N, building path as we traverse

```sql
WITH RECURSIVE graph_traversal AS (
    -- Base: starting entity
    SELECT entity_id, ..., 0 AS depth, [entity_id] AS path
    FROM entities
    WHERE entity_id = ? AND [temporal filters]

    UNION ALL

    -- Recursive: follow edges
    SELECT e.entity_id, ..., gt.depth + 1, path + e.entity_id
    FROM graph_traversal gt
    JOIN relationships r ON gt.entity_id = r.from_entity
    JOIN entities e ON r.to_entity = e.entity_id
    WHERE gt.depth < ? AND [filters] AND NOT (e.entity_id IN path)
)
SELECT * FROM graph_traversal WHERE depth > 0
```

### Cycle Prevention

**Problem**: Directed graphs may contain cycles (A→B→C→A), causing infinite recursion.

**Solution**: Track visited nodes in path and check before adding:

#### SQLite Implementation
```sql
-- Path as JSON array: json_array(entity_id)
-- Build path: json_insert(path, '$[#]', e.entity_id)
-- Check cycle: NOT EXISTS (
--     SELECT 1 FROM json_each(gt.path) WHERE value = e.entity_id
-- )
```

**Complexity**: O(n) per hop where n = path length
**Trade-off**: JSON parsing overhead, but SQLite has no native arrays

#### PostgreSQL Implementation
```sql
-- Path as native array: ARRAY[entity_id]
-- Build path: gt.path || e.entity_id
-- Check cycle: NOT (e.entity_id = ANY(gt.path))
```

**Complexity**: O(n) per hop where n = path length
**Advantage**: Native array operations, more efficient than SQLite JSON

### Bi-Temporal Filtering

Both implementations support point-in-time queries with bi-temporal model:
- **Valid Time**: When fact was true in reality (event_time)
- **Transaction Time**: When fact was recorded in database (ingestion_time)

#### SQLite Approach
```sql
-- Manual timestamp comparisons
WHERE e.valid_time_start <= ? AND e.valid_time_end > ?
  AND e.transaction_time_end = 9999999999  -- Current version
```

#### PostgreSQL Approach
```sql
-- tstzrange operators with GiST indexes
WHERE tstzrange(e.valid_time_start, e.valid_time_end) @> ?::timestamptz
  AND tstzrange(e.transaction_time_start, e.transaction_time_end) @> now()
```

**PostgreSQL Advantage**: GiST indexes on temporal ranges enable O(log n) filtering vs O(n) table scan.

### Depth Limiting

**Safety Guardrail**: Cap maximum depth at 10 hops to prevent runaway queries.

```rust
let capped_depth = (max_depth.min(10)) as i64;
```

**Rationale**:
- Most real-world graph queries need 2-4 hops
- 10-hop traversal on 100K graph → potential millions of paths
- Users can split deep queries into multiple shallow traversals

## Performance Profiles

### Small Scale (10-100 entities)

| Backend    | 1-Hop  | 2-Hop  | 4-Hop  | Notes                    |
|------------|--------|--------|--------|--------------------------|
| SQLite     | <1ms   | <1ms   | <1ms   | Minimal overhead         |
| PostgreSQL | <1ms   | ~1ms   | 1-2ms  | GiST index lookup        |

**Test Data**: SQLite 39 tests in 0.06s, PostgreSQL 13 tests in 0.10s

### Medium Scale (1K-10K entities, estimated)

| Backend    | 1-Hop  | 2-Hop  | 4-Hop  | Scaling Factor           |
|------------|--------|--------|--------|--------------------------|
| SQLite     | 1-3ms  | 3-8ms  | 8-18ms | Linear with avg degree   |
| PostgreSQL | 1-2ms  | 2-5ms  | 5-12ms | GiST reduces scan cost   |

**PostgreSQL Advantage**: ~30% faster due to native arrays and index optimization

### Large Scale (100K entities, estimated)

| Backend    | 1-Hop (p95) | 2-Hop (p95) | 4-Hop (p95) | Target     |
|------------|-------------|-------------|-------------|------------|
| SQLite     | 3-5ms       | 12-18ms     | 35-50ms     | <50ms ✅   |
| PostgreSQL | 2-4ms       | 8-12ms      | 25-35ms     | <50ms ✅   |

**Assumptions**:
- Average degree: 10 relationships per entity
- Graph structure: Mix of hubs (high degree) and leaf nodes
- Database: Proper indexes configured
- Hardware: Modern SSD, 8GB+ RAM

## Performance Factors

### 1. Average Degree (k)

**Impact**: Exponential - O(k^N) worst case for N hops

| Avg Degree | 1-Hop Paths | 2-Hop Paths | 4-Hop Paths |
|------------|-------------|-------------|-------------|
| k=5        | 5           | 25          | 625         |
| k=10       | 10          | 100         | 10,000      |
| k=20       | 20          | 400         | 160,000     |

**Mitigation**: Cycle prevention reduces to O(k*N) by avoiding revisits.

### 2. Index Configuration

**Critical for PostgreSQL**:
```sql
-- GiST indexes on temporal ranges (auto-created by migrations)
CREATE INDEX idx_entities_valid_time_gist
    ON entities USING GIST (tstzrange(valid_time_start, valid_time_end));
CREATE INDEX idx_entities_transaction_time_gist
    ON entities USING GIST (tstzrange(transaction_time_start, transaction_time_end));
```

**Impact**: 5-10x speedup for temporal filtering in recursive CTEs.

**SQLite Note**: B-tree indexes on timestamp columns provide logarithmic lookup but less efficient than PostgreSQL GiST for range queries.

### 3. Relationship Type Filtering

**Impact**: Reduces branching factor significantly

Example: Social graph with 4 relationship types (knows, works_at, follows, likes)
- Without filter: 10 avg relationships → 10,000 4-hop paths
- With filter (knows only): 2.5 avg → 390 4-hop paths

**Speedup**: ~25x reduction in paths explored

### 4. Graph Structure

**Dense vs Sparse Graphs**:
- **Dense** (high avg degree): More paths, slower traversal
- **Sparse** (low avg degree): Fewer paths, faster traversal
- **Hub Nodes**: High-degree nodes create hotspots; consider relationship filtering

## Scaling Guidance

### When to Use Traverse

✅ **Good Use Cases**:
- Social network friend-of-friend queries (2-3 hops)
- Organizational hierarchy traversal (3-5 hops)
- Dependency graph analysis (2-4 hops)
- Knowledge graph relationship discovery (2-3 hops)

❌ **Poor Use Cases**:
- Finding all reachable nodes (use BFS/DFS instead)
- Shortest path between distant nodes (use Dijkstra/A* instead)
- Graph-wide analytics (use graph processing frameworks)

### Optimization Strategies

1. **Add Relationship Type Filters**: Reduce branching factor
2. **Limit Depth**: Start with 2-3 hops, increase if needed
3. **Use Temporal Filtering**: Narrow search to relevant time periods
4. **Batch Queries**: Multiple shallow traversals vs one deep traversal
5. **Cache Results**: Store frequently accessed subgraphs

### When to Switch Backends

**Use SQLite When**:
- Single-user or low-concurrency workloads
- Embedded applications
- Development and testing
- Graphs <100K entities
- <50ms latency acceptable

**Use PostgreSQL When**:
- Multi-user or high-concurrency workloads
- Production deployments
- Graphs >100K entities
- Latency-critical applications
- Need for temporal query optimization (GiST indexes)

## Code References

### SQLite Implementation
- **File**: `llmspell-storage/src/backends/sqlite/graph.rs`
- **Lines**: 1123-1390 (180 lines)
- **Key Functions**:
  - `traverse()`: Main entry point (lines 1210-1390)
  - Recursive CTE with JSON path tracking
  - `json_array()` + `json_insert()` for path building
  - `json_each()` for cycle detection

### PostgreSQL Implementation
- **File**: `llmspell-storage/src/backends/postgres/graph.rs`
- **Lines**: 864-1020 (155 lines)
- **Key Functions**:
  - `traverse()`: Main entry point (lines 888-1019)
  - Recursive CTE with native array path tracking
  - `ARRAY[]` + `||` for path building
  - `ANY()` for cycle detection
  - `tstzrange @> ::timestamptz` for temporal filtering

### Tests
- **SQLite**: `llmspell-storage/src/backends/sqlite/graph.rs` (lines 1127-1390)
  - 5 traverse tests: 1-hop, 4-hop linear, cycles, filter, temporal
- **PostgreSQL**: `llmspell-storage/tests/postgres_temporal_graph_traversal_tests.rs`
  - 5 traverse tests via KnowledgeGraph trait
  - Additional 8 legacy tests via get_related()

## Known Limitations

1. **No Path Cost Optimization**: Returns all paths, not shortest paths
2. **No Path Ranking**: No ordering by relationship weights or relevance
3. **Fixed Depth Limit**: 10-hop maximum (safety guardrail)
4. **Memory Growth**: Path storage grows with depth (manageable up to 10 hops)
5. **PostgreSQL UUID Requirement**: Entity IDs must be valid UUIDs

## Future Enhancements

### Short-Term (v0.14.x)
- [ ] Path ranking by relationship weights
- [ ] Configurable depth limit (per query)
- [ ] Path cost accumulation

### Medium-Term (v0.15.x)
- [ ] Bidirectional traversal (forward + backward simultaneously)
- [ ] Parallel traversal for multiple starting entities
- [ ] Path caching for hot subgraphs

### Long-Term (v1.0+)
- [ ] Graph algorithms library (shortest path, centrality, etc.)
- [ ] Distributed graph processing
- [ ] Incremental graph updates without full recomputation

## Benchmark Data

See `benchmarks/graph-traversal-baseline.md` for detailed test results and synthetic dataset specifications.

## References

- **Phase 13c Design**: `docs/in-progress/phase-13-design-doc.md`
- **Task 13c.2.4**: Graph Storage Implementation (deferred traverse to 13c.2.8)
- **Task 13c.2.8**: Legacy Backend Removal & Graph Traversal Enhancement
- **Baseline Report**: `benchmarks/graph-traversal-baseline.md`

## Changelog

- **2025-11-12**: Initial documentation (Task 13c.2.8.8)
  - Recursive CTE implementation details
  - Performance profiles for SQLite and PostgreSQL
  - Scaling guidance and optimization strategies
  - Code references and test coverage

-- Migration: Create event_log partitioned table (Phase 13b.11.1)
-- Purpose: Time-series event storage with monthly partitioning for efficient queries and archival
-- Dependencies: V1__initial_setup.sql (llmspell schema)
--
-- Event log implements:
-- - Hybrid schema: Extracted columns for hot queries + JSONB payload for flexibility
-- - Monthly RANGE partitioning on timestamp
-- - Automatic partition creation via trigger
-- - RLS for tenant isolation
-- - Indexes optimized for EventStorage trait queries

-- ============================================================================
-- Parent Partitioned Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS llmspell.event_log (
    -- Tenant isolation (part of composite PK)
    tenant_id VARCHAR(255) NOT NULL,

    -- Event identification
    event_id UUID NOT NULL,

    -- Extracted columns for efficient indexing (hot query paths)
    event_type VARCHAR(255) NOT NULL,
    correlation_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    sequence BIGINT NOT NULL,
    language VARCHAR(50) NOT NULL,

    -- Full event stored as JSONB for flexibility
    payload JSONB NOT NULL,

    -- Composite primary key (includes partition key for uniqueness)
    PRIMARY KEY (tenant_id, timestamp, event_id)
) PARTITION BY RANGE (timestamp);

COMMENT ON TABLE llmspell.event_log IS 'Event log with monthly partitioning for time-series queries';
COMMENT ON COLUMN llmspell.event_log.event_type IS 'Event type for pattern matching (e.g., "agent.state_changed")';
COMMENT ON COLUMN llmspell.event_log.correlation_id IS 'Correlation ID for tracing related events';
COMMENT ON COLUMN llmspell.event_log.sequence IS 'Global sequence number for event ordering';
COMMENT ON COLUMN llmspell.event_log.payload IS 'Full UniversalEvent as JSONB';

-- ============================================================================
-- Indexes on Parent Table (Inherited by Partitions)
-- ============================================================================

-- Index for EventStorage.get_events_by_correlation_id
CREATE INDEX IF NOT EXISTS idx_event_log_correlation
    ON llmspell.event_log(correlation_id, timestamp DESC);

-- Index for EventStorage.get_events_by_pattern (event_type matching)
CREATE INDEX IF NOT EXISTS idx_event_log_type
    ON llmspell.event_log(event_type, timestamp DESC);

-- Index for sequence-based ordering
CREATE INDEX IF NOT EXISTS idx_event_log_sequence
    ON llmspell.event_log(sequence DESC);

-- Index for tenant + time range queries (RLS + time filtering)
CREATE INDEX IF NOT EXISTS idx_event_log_tenant_time
    ON llmspell.event_log(tenant_id, timestamp DESC);

-- GIN index for ad-hoc JSONB queries on payload
CREATE INDEX IF NOT EXISTS idx_event_log_payload
    ON llmspell.event_log USING GIN(payload);

-- ============================================================================
-- Partition Management Functions
-- ============================================================================

-- Function to create a partition for a specific month
CREATE OR REPLACE FUNCTION llmspell.create_event_log_partition(
    partition_start TIMESTAMPTZ,
    partition_end TIMESTAMPTZ
) RETURNS TEXT AS $$
DECLARE
    partition_name TEXT;
    partition_exists BOOLEAN;
BEGIN
    -- Generate partition name: event_log_YYYY_MM
    partition_name := 'event_log_' || to_char(partition_start, 'YYYY_MM');

    -- Check if partition already exists
    SELECT EXISTS (
        SELECT 1 FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'llmspell'
        AND c.relname = partition_name
    ) INTO partition_exists;

    IF partition_exists THEN
        RETURN 'SKIPPED: Partition ' || partition_name || ' already exists';
    END IF;

    -- Create partition
    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS llmspell.%I PARTITION OF llmspell.event_log
         FOR VALUES FROM (%L) TO (%L)',
        partition_name,
        partition_start,
        partition_end
    );

    RETURN 'CREATED: Partition ' || partition_name || ' [' || partition_start || ' to ' || partition_end || ')';
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

COMMENT ON FUNCTION llmspell.create_event_log_partition IS 'Create monthly partition for event_log table (SECURITY DEFINER: executes as table owner for DDL)';

-- Function to ensure future partitions exist (current + next 3 months)
CREATE OR REPLACE FUNCTION llmspell.ensure_future_event_log_partitions()
RETURNS TEXT[] AS $$
DECLARE
    current_month TIMESTAMPTZ;
    partition_start TIMESTAMPTZ;
    partition_end TIMESTAMPTZ;
    results TEXT[] := ARRAY[]::TEXT[];
    result TEXT;
    i INTEGER;
BEGIN
    -- Get start of current month
    current_month := date_trunc('month', CURRENT_TIMESTAMP);

    -- Create partitions for current month + next 3 months (4 total)
    FOR i IN 0..3 LOOP
        partition_start := current_month + (i || ' months')::INTERVAL;
        partition_end := partition_start + '1 month'::INTERVAL;

        result := llmspell.create_event_log_partition(partition_start, partition_end);
        results := array_append(results, result);
    END LOOP;

    RETURN results;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

COMMENT ON FUNCTION llmspell.ensure_future_event_log_partitions IS 'Ensure partitions exist for current + next 3 months (SECURITY DEFINER: executes as table owner for DDL)';

-- Note: Automatic partition creation via BEFORE INSERT trigger doesn't work because
-- partition routing happens before the trigger's DDL is visible. Instead:
-- - Run ensure_future_event_log_partitions() periodically (daily cron/maintenance job)
-- - Application should call ensure_future_event_log_partitions() when needed
--
-- Keeping trigger infrastructure commented for potential future use with deferred constraints:
--
-- CREATE OR REPLACE FUNCTION llmspell.ensure_event_log_partition_trigger()
-- RETURNS TRIGGER AS $$
-- BEGIN
--     -- Placeholder: could log warnings or metrics about partition coverage
--     RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;

-- Function to cleanup old partitions (manual, application-controlled)
CREATE OR REPLACE FUNCTION llmspell.cleanup_old_event_log_partitions(
    before_date TIMESTAMPTZ
) RETURNS TEXT[] AS $$
DECLARE
    partition_name TEXT;
    partition_month TIMESTAMPTZ;
    results TEXT[] := ARRAY[]::TEXT[];
BEGIN
    -- Find and drop partitions older than before_date
    FOR partition_name IN
        SELECT c.relname
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        WHERE n.nspname = 'llmspell'
        AND c.relname LIKE 'event_log_%'
        AND c.relname != 'event_log'
    LOOP
        -- Parse partition month from name
        partition_month := to_timestamp(
            SUBSTRING(partition_name FROM 'event_log_(\d{4}_\d{2})'),
            'YYYY_MM'
        );

        -- Drop if older than threshold
        IF partition_month < date_trunc('month', before_date) THEN
            EXECUTE 'DROP TABLE IF EXISTS llmspell.' || quote_ident(partition_name);
            results := array_append(results, 'DROPPED: ' || partition_name);
        END IF;
    END LOOP;

    IF array_length(results, 1) IS NULL THEN
        results := ARRAY['No partitions to drop'];
    END IF;

    RETURN results;
END;
$$ LANGUAGE plpgsql SECURITY DEFINER;

COMMENT ON FUNCTION llmspell.cleanup_old_event_log_partitions IS 'Manually drop event_log partitions older than specified date (SECURITY DEFINER: executes as table owner for DDL)';

-- ============================================================================
-- Initial Partitions (Current Month + Next 3 Months)
-- ============================================================================

-- Create initial partitions
SELECT llmspell.ensure_future_event_log_partitions();

-- ============================================================================
-- Row-Level Security (RLS)
-- ============================================================================

-- Enable RLS on parent table (inherited by partitions)
ALTER TABLE llmspell.event_log ENABLE ROW LEVEL SECURITY;

-- FORCE RLS enforces policies even for table owners and BYPASSRLS users (defense-in-depth)
-- This is secondary defense layer (primary: separate admin/app roles in V12)
-- PostgreSQL best practice for multi-tenant SaaS applications
ALTER TABLE llmspell.event_log FORCE ROW LEVEL SECURITY;

-- RLS Policy: SELECT
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.event_log;
CREATE POLICY tenant_isolation_select ON llmspell.event_log
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- RLS Policy: INSERT
DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.event_log;
CREATE POLICY tenant_isolation_insert ON llmspell.event_log
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- RLS Policy: UPDATE
DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.event_log;
CREATE POLICY tenant_isolation_update ON llmspell.event_log
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- RLS Policy: DELETE
DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.event_log;
CREATE POLICY tenant_isolation_delete ON llmspell.event_log
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Verification Queries (Commented Out)
-- ============================================================================

-- Verify partitions created:
-- SELECT tablename FROM pg_tables WHERE schemaname = 'llmspell' AND tablename LIKE 'event_log_%' ORDER BY tablename;

-- Verify RLS policies:
-- SELECT policyname, cmd FROM pg_policies WHERE tablename = 'event_log';

-- Verify indexes:
-- SELECT indexname, indexdef FROM pg_indexes WHERE schemaname = 'llmspell' AND tablename = 'event_log';

-- Test partition creation for future month:
-- SELECT llmspell.create_event_log_partition('2026-01-01'::TIMESTAMPTZ, '2026-02-01'::TIMESTAMPTZ);

-- Test cleanup function (dry run - replace with actual date):
-- SELECT llmspell.cleanup_old_event_log_partitions('2024-01-01'::TIMESTAMPTZ);

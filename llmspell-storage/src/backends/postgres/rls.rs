//! ABOUTME: Row-Level Security (RLS) policy generation helpers (Phase 13b.3.1)
//! ABOUTME: SQL generation for applying tenant isolation policies to PostgreSQL tables

/// Generate RLS policy SQL for a given table
///
/// Creates four policies for complete tenant isolation:
/// - SELECT: Users can only read rows matching their tenant_id
/// - INSERT: Users can only insert rows with their tenant_id
/// - UPDATE: Users can only update rows matching their tenant_id
/// - DELETE: Users can only delete rows matching their tenant_id
///
/// # Arguments
/// * `table_name` - Name of the table (without schema prefix)
///
/// # Returns
/// * `String` - SQL statements to enable RLS and create policies
///
/// # Example
/// ```rust
/// use llmspell_storage::backends::postgres::rls::generate_rls_policies;
///
/// let sql = generate_rls_policies("vector_embeddings");
/// assert!(sql.contains("ENABLE ROW LEVEL SECURITY"));
/// assert!(sql.contains("tenant_isolation_select"));
/// ```
///
/// # Security
/// The generated SQL uses `current_setting('app.current_tenant_id', true)` to enforce
/// tenant isolation. This requires the application to call `set_tenant_context()` before
/// querying tables with RLS enabled.
///
/// # Note
/// All policies are created with `IF NOT EXISTS` for idempotency.
pub fn generate_rls_policies(table_name: &str) -> String {
    format!(
        r#"
-- Enable RLS on {table}
ALTER TABLE llmspell.{table} ENABLE ROW LEVEL SECURITY;

-- SELECT policy: Only see rows for current tenant
CREATE POLICY IF NOT EXISTS tenant_isolation_select ON llmspell.{table}
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert rows for current tenant
CREATE POLICY IF NOT EXISTS tenant_isolation_insert ON llmspell.{table}
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update rows for current tenant
CREATE POLICY IF NOT EXISTS tenant_isolation_update ON llmspell.{table}
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete rows for current tenant
CREATE POLICY IF NOT EXISTS tenant_isolation_delete ON llmspell.{table}
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));
"#,
        table = table_name
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rls_policies_contains_enable_statement() {
        let sql = generate_rls_policies("test_table");
        assert!(
            sql.contains("ALTER TABLE llmspell.test_table ENABLE ROW LEVEL SECURITY"),
            "SQL should enable RLS"
        );
    }

    #[test]
    fn test_generate_rls_policies_contains_all_four_policies() {
        let sql = generate_rls_policies("test_table");

        assert!(
            sql.contains("tenant_isolation_select"),
            "Should have SELECT policy"
        );
        assert!(
            sql.contains("tenant_isolation_insert"),
            "Should have INSERT policy"
        );
        assert!(
            sql.contains("tenant_isolation_update"),
            "Should have UPDATE policy"
        );
        assert!(
            sql.contains("tenant_isolation_delete"),
            "Should have DELETE policy"
        );
    }

    #[test]
    fn test_generate_rls_policies_uses_if_not_exists() {
        let sql = generate_rls_policies("test_table");

        // Count occurrences of IF NOT EXISTS (should be 4, one per policy)
        let count = sql.matches("IF NOT EXISTS").count();
        assert_eq!(
            count, 4,
            "Should have IF NOT EXISTS for all 4 policies (idempotency)"
        );
    }

    #[test]
    fn test_generate_rls_policies_uses_current_setting() {
        let sql = generate_rls_policies("test_table");

        // All policies should use current_setting for tenant_id
        // SELECT: 1 (USING), INSERT: 1 (CHECK), UPDATE: 2 (USING+CHECK), DELETE: 1 (USING) = 5 total
        let count = sql.matches("current_setting('app.current_tenant_id', true)").count();
        assert_eq!(
            count, 5,
            "Should use current_setting 5 times (SELECT USING, INSERT CHECK, UPDATE USING+CHECK, DELETE USING)"
        );
    }

    #[test]
    fn test_generate_rls_policies_includes_schema_prefix() {
        let sql = generate_rls_policies("my_table");

        // Should use llmspell schema
        assert!(
            sql.contains("llmspell.my_table"),
            "Should use llmspell schema prefix"
        );
    }

    #[test]
    fn test_generate_rls_policies_update_has_both_using_and_check() {
        let sql = generate_rls_policies("test_table");

        // UPDATE policy should have both USING and WITH CHECK
        assert!(
            sql.contains("FOR UPDATE") && sql.contains("USING") && sql.contains("WITH CHECK"),
            "UPDATE policy should have both USING and WITH CHECK clauses"
        );
    }

    #[test]
    fn test_generate_rls_policies_with_different_table_names() {
        let tables = vec!["vector_embeddings", "entities", "relationships"];

        for table in tables {
            let sql = generate_rls_policies(table);
            assert!(
                sql.contains(&format!("llmspell.{}", table)),
                "Should work with table name: {}",
                table
            );
        }
    }
}

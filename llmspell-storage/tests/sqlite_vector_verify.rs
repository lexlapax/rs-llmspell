use anyhow::Result;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};

#[tokio::test]
async fn test_vector_extension_loaded() -> Result<()> {
    // 1. Initialize backend (which loads extension)
    let config = SqliteConfig::in_memory();
    let backend = SqliteBackend::new(config).await?;
    let conn = backend.get_connection().await?;

    // 2. Verify module is loaded
    // Move connection usage to a blocking task or use it directly since tests can block?
    // Actually, conn is MutexGuard or PooledConnection. rusqlite operations are blocking.
    // In tokio test, we can just call them.

    let found = conn
        .query_row(
            "SELECT name FROM pragma_module_list WHERE name = 'vectorlite'",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);

    assert!(found, "vectorlite module not found in pragma_module_list");

    // 3. Verify Basic Vector Operations
    // Create a virtual table using vectorlite
    // Create a virtual table using vectorlite
    conn.execute(
        "CREATE VIRTUAL TABLE v_test USING vectorlite(dimension=3, metric='l2', max_elements=100)",
        [],
    )?;

    // Insert a vector
    conn.execute(
        "INSERT INTO v_test(rowid, embedding) VALUES (1, '[1.0, 2.0, 3.0]')",
        [],
    )?;

    // Search
    let mut stmt = conn.prepare(
        "SELECT rowid, distance FROM v_test WHERE embedding MATCH '[1.0, 2.0, 3.0]' ORDER BY distance LIMIT 1"
    )?;

    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        let rowid: i64 = row.get(0)?;
        let distance: f64 = row.get(1)?;
        assert_eq!(rowid, 1);
        assert!(distance < 1e-5);
    } else {
        panic!("No results found from vector search");
    }

    Ok(())
}

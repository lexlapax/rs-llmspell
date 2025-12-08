use anyhow::Result;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};

#[tokio::test]
async fn test_vector_extension_loaded() -> Result<()> {
    // 1. Initialize backend (which loads extension)
    let config = SqliteConfig::in_memory();
    let backend = SqliteBackend::new(config).await?;
    let conn = backend.get_connection().await?;

    // 2. Verify module is loaded
    let mut stmt = conn
        .prepare("SELECT name FROM pragma_module_list WHERE name = 'vectorlite'")
        .await?;
    let mut rows = stmt.query(()).await?;

    let mut found = false;
    while let Some(row) = rows.next().await? {
        let name: String = row.get(0)?;
        if name == "vectorlite" {
            found = true;
        }
    }

    assert!(found, "vectorlite module not found in pragma_module_list");

    // 3. Verify Basic Vector Operations
    // Create a virtual table using vectorlite
    conn.execute(
        "CREATE VIRTUAL TABLE v_test USING vectorlite(embedding float32[3], hnsw(max_elements=100))",
        (),
    ).await?;

    // Insert a vector
    conn.execute(
        "INSERT INTO v_test(rowid, embedding) VALUES (1, '[1.0, 2.0, 3.0]')",
        (),
    )
    .await?;

    // Search
    let mut stmt = conn.prepare(
        "SELECT rowid, distance FROM v_test WHERE knn_search(embedding, 3, '[1.0, 2.0, 3.0]') ORDER BY distance LIMIT 1"
    ).await?;

    let mut rows = stmt.query(()).await?;
    if let Some(row) = rows.next().await? {
        let rowid: i64 = row.get(0)?;
        let distance: f64 = row.get(1)?;
        assert_eq!(rowid, 1);
        assert!(distance < 1e-5);
    } else {
        panic!("No results found from vector search");
    }

    Ok(())
}

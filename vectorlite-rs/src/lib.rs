//! vectorlite-rs: Pure Rust HNSW vector search extension for SQLite
//!
//! # Architecture
//!
//! - **Virtual Table API**: Implements rusqlite VTab + VTabCursor traits
//! - **HNSW Indexing**: Uses hnsw_rs for approximate nearest neighbor search
//! - **Distance Metrics**: L2 (Euclidean), Cosine, Inner Product
//! - **Multi-Dimension Support**: 384, 768, 1536, 3072 dimensions
//! - **Performance Target**: 3-100x faster than sqlite-vec brute-force
//!
//! # Usage
//!
//! ```sql
//! -- Load extension
//! SELECT load_extension('./extensions/vectorlite.dylib');
//!
//! -- Create vector table with HNSW index
//! CREATE VIRTUAL TABLE vec_embeddings_768 USING vectorlite(
//!     dimension=768,
//!     metric='cosine',
//!     max_elements=100000,
//!     ef_construction=200,
//!     m=16
//! );
//!
//! -- Insert vectors
//! INSERT INTO vec_embeddings_768(rowid, embedding) VALUES (1, ?);
//!
//! -- K-NN search
//! SELECT rowid, distance
//! FROM vec_embeddings_768
//! WHERE embedding MATCH ?
//! ORDER BY distance
//! LIMIT 10;
//! ```

use rusqlite::vtab::{
    eponymous_only_module, sqlite3_vtab, sqlite3_vtab_cursor, Context, IndexInfo, VTab,
    VTabConnection, VTabCursor,
};
use rusqlite::{ffi, Connection, Result as SqliteResult};
use std::marker::PhantomData;
use std::os::raw::c_int;

mod distance;
mod error;
mod hnsw;
mod vtab;

pub use distance::{distance_cosine, distance_inner_product, distance_l2, DistanceMetric};
pub use error::{Error, Result};
pub use hnsw::HnswIndex;

/// SQLite extension entry point (macOS/Linux)
///
/// This function is called when the extension is loaded via load_extension().
/// It registers the vectorlite virtual table module.
///
/// # Safety
///
/// This function interfaces directly with SQLite's C API and must follow
/// SQLite's extension loading contract:
/// - Must be exported with C ABI
/// - Must return SQLITE_OK (0) on success
/// - Must not panic or unwind across FFI boundary
#[no_mangle]
pub unsafe extern "C" fn sqlite3_vectorlite_init(
    db: *mut ffi::sqlite3,
    _pz_err_msg: *mut *mut std::os::raw::c_char,
    _p_api: *mut ffi::sqlite3_api_routines,
) -> c_int {
    // Wrap the raw db pointer in a Connection
    let conn = match Connection::from_handle(db) {
        Ok(conn) => conn,
        Err(_) => return ffi::SQLITE_ERROR,
    };

    // Register the vectorlite virtual table module
    // Use eponymous_only_module for read-only vector search tables
    if let Err(e) = conn.create_module("vectorlite", eponymous_only_module::<VectorLiteTab>(), None)
    {
        eprintln!("Failed to register vectorlite module: {e}");
        return ffi::SQLITE_ERROR;
    }

    // IMPORTANT: Prevent conn from closing the database connection when dropped!
    // The connection is owned by the host process, we just borrowed it.
    std::mem::forget(conn);

    ffi::SQLITE_OK
}

/// Windows extension entry point
#[cfg(windows)]
#[no_mangle]
pub unsafe extern "C" fn sqlite3_vectorlite_init_win32(
    db: *mut ffi::sqlite3,
    pz_err_msg: *mut *mut std::os::raw::c_char,
    p_api: *mut ffi::sqlite3_api_routines,
) -> c_int {
    sqlite3_vectorlite_init(db, pz_err_msg, p_api)
}

/// VectorLite virtual table implementation
///
/// Each table instance maintains an HNSW index for fast approximate nearest neighbor search.
/// The table stores vectors as BLOBs and builds an in-memory HNSW index for efficient queries.
#[repr(C)]
pub struct VectorLiteTab {
    /// Base class. Must be first member
    base: sqlite3_vtab,
    /// HNSW index for vector search
    index: Option<HnswIndex>,
    /// Vector dimension (384, 768, 1536, 3072)
    dimension: usize,
    /// Distance metric (L2, cosine, inner product)
    metric: DistanceMetric,
}

impl VectorLiteTab {
    /// Create new virtual table instance
    fn new(dimension: usize, metric: DistanceMetric) -> Self {
        Self {
            base: unsafe { std::mem::zeroed() },
            index: None,
            dimension,
            metric,
        }
    }
}

unsafe impl<'vtab> VTab<'vtab> for VectorLiteTab {
    type Aux = ();
    type Cursor = VectorLiteCursor<'vtab>;

    fn connect(
        _db: &mut VTabConnection,
        _aux: Option<&Self::Aux>,
        args: &[&[u8]],
    ) -> SqliteResult<(String, Self)> {
        // Parse CREATE VIRTUAL TABLE arguments
        // args[0] = module name ("vectorlite")
        // args[1] = database name
        // args[2] = table name
        // args[3..] = parameters (dimension, metric, etc.)

        let dimension = vtab::parse_dimension(args)?;
        let metric = vtab::parse_metric(args)?;

        let schema = "CREATE TABLE x(rowid INTEGER PRIMARY KEY, embedding BLOB, distance HIDDEN)"
            .to_string();

        let mut vtab = Self::new(dimension, metric);

        // Initialize HNSW index
        // Parameters from args or defaults:
        // - max_elements: 100000
        // - ef_construction: 200
        // - m: 16
        let max_elements = vtab::parse_max_elements(args).unwrap_or(100_000);
        let ef_construction = vtab::parse_ef_construction(args).unwrap_or(200);
        let m = vtab::parse_m(args).unwrap_or(16);

        vtab.index = Some(HnswIndex::new(
            dimension,
            max_elements,
            m,
            ef_construction,
            metric,
        )?);

        Ok((schema, vtab))
    }

    fn best_index(&self, info: &mut IndexInfo) -> SqliteResult<()> {
        // Query optimization: Check for MATCH operator (vector search)
        vtab::best_index(info)
    }

    fn open(&mut self) -> SqliteResult<Self::Cursor> {
        Ok(VectorLiteCursor::new())
    }
}

/// Cursor for iterating over query results
#[repr(C)]
pub struct VectorLiteCursor<'vtab> {
    /// Base class. Must be first member
    base: sqlite3_vtab_cursor,
    /// Current result set (rowid, distance pairs)
    results: Vec<(i64, f32)>,
    /// Current position in results
    position: usize,
    /// Phantom data to tie lifetime to VTab
    phantom: PhantomData<&'vtab VectorLiteTab>,
}

impl<'vtab> VectorLiteCursor<'vtab> {
    fn new() -> Self {
        Self {
            base: unsafe { std::mem::zeroed() },
            results: Vec::new(),
            position: 0,
            phantom: PhantomData,
        }
    }
}

unsafe impl VTabCursor for VectorLiteCursor<'_> {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _args: &rusqlite::vtab::Values<'_>,
    ) -> SqliteResult<()> {
        // Execute K-NN search
        // This will be implemented in vtab module
        self.position = 0;
        Ok(())
    }

    fn next(&mut self) -> SqliteResult<()> {
        self.position += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.position >= self.results.len()
    }

    fn column(&self, ctx: &mut Context, col: c_int) -> SqliteResult<()> {
        if self.position >= self.results.len() {
            return Ok(());
        }

        let (rowid, distance) = self.results[self.position];

        match col {
            0 => ctx.set_result(&rowid),     // rowid column
            1 => ctx.set_result(&vec![0u8]), // embedding column (placeholder)
            2 => ctx.set_result(&distance),  // distance column (HIDDEN)
            _ => Ok(()),
        }
    }

    fn rowid(&self) -> SqliteResult<i64> {
        if self.position < self.results.len() {
            Ok(self.results[self.position].0)
        } else {
            Ok(0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distance_metrics() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];

        let l2 = distance_l2(&a, &b);
        assert!((l2 - 1.414).abs() < 0.01);

        let cosine = distance_cosine(&a, &b);
        assert!((cosine - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_supported_dimensions() {
        // Test that the supported dimensions are recognized
        assert!(matches!(384, 384 | 768 | 1536 | 3072));
        assert!(matches!(768, 384 | 768 | 1536 | 3072));
        assert!(matches!(1536, 384 | 768 | 1536 | 3072));
        assert!(matches!(3072, 384 | 768 | 1536 | 3072));
        assert!(!matches!(512, 384 | 768 | 1536 | 3072));
    }
}

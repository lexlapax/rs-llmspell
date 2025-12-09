use rusqlite::ffi;
use rusqlite::{Connection, Result as SqliteResult};
use std::ffi::{CStr, CString};
use std::mem;
use std::os::raw::{c_char, c_int, c_void};
use std::ptr;
use std::slice;

mod distance;
mod error;
mod hnsw;
mod vtab;

pub use distance::{distance_cosine, distance_inner_product, distance_l2, DistanceMetric};
pub use error::{Error, Result};
pub use hnsw::HnswIndex;

/// Programmatic entry point for static linking.
pub fn register_vectorlite(conn: &Connection) -> SqliteResult<()> {
    static MODULE: ffi::sqlite3_module = ffi::sqlite3_module {
        iVersion: 0,
        xCreate: Some(x_create),
        xConnect: Some(x_connect),
        xBestIndex: Some(x_best_index),
        xDisconnect: Some(x_disconnect),
        xDestroy: Some(x_destroy),
        xOpen: Some(x_open),
        xClose: Some(x_close),
        xFilter: Some(x_filter),
        xNext: Some(x_next),
        xEof: Some(x_eof),
        xColumn: Some(x_column),
        xRowid: Some(x_rowid),
        xUpdate: Some(x_update),
        xBegin: None,
        xSync: None,
        xCommit: None,
        xRollback: None,
        xFindFunction: None,
        xRename: None,
        xSavepoint: None,
        xRelease: None,
        xRollbackTo: None,
        xShadowName: None,
        xIntegrity: None,
    };

    unsafe {
        let db = conn.handle();
        let c_name = CString::new("vectorlite").unwrap();
        let rc = ffi::sqlite3_create_module_v2(db, c_name.as_ptr(), &MODULE, ptr::null_mut(), None);
        if rc != ffi::SQLITE_OK {
            return Err(rusqlite::Error::SqliteFailure(
                ffi::Error {
                    code: ffi::ErrorCode::Unknown,
                    extended_code: rc,
                },
                Some("Failed to create module vectorlite".to_string()),
            ));
        }
    }
    Ok(())
}

/// VectorLite virtual table structure (C compatible)
#[repr(C)]
pub struct VectorLiteTab {
    base: ffi::sqlite3_vtab,
    index: Option<HnswIndex>,
    dimension: usize,
    metric: DistanceMetric,
}

/// VectorLite cursor structure (C compatible)
#[repr(C)]
pub struct VectorLiteCursor {
    base: ffi::sqlite3_vtab_cursor,
    results: Vec<(i64, f32)>,
    position: usize,
}

// --- C Callbacks ---

unsafe extern "C" fn x_connect(
    db: *mut ffi::sqlite3,
    _aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut ffi::sqlite3_vtab,
    _pz_err: *mut *mut c_char,
) -> c_int {
    // Parse args
    let args_slice = slice::from_raw_parts(argv, argc as usize);
    let mut args_bytes = Vec::with_capacity(argc as usize);
    for &arg in args_slice {
        args_bytes.push(CStr::from_ptr(arg).to_bytes());
    }

    // Call parsing logic (reuse vtab helpers)
    let dimension_res = vtab::parse_dimension(&args_bytes);
    let metric_res = vtab::parse_metric(&args_bytes);

    let dimension = match dimension_res {
        Ok(d) => d,
        Err(_) => return ffi::SQLITE_ERROR,
    };
    let metric = match metric_res {
        Ok(m) => m,
        Err(_) => return ffi::SQLITE_ERROR,
    };

    // Declare table
    // Removed explicit rowid column - it's implicit in virtual tables
    let schema = CString::new("CREATE TABLE x(embedding BLOB, distance HIDDEN)").unwrap();
    if ffi::sqlite3_declare_vtab(db, schema.as_ptr()) != ffi::SQLITE_OK {
        return ffi::SQLITE_ERROR;
    }

    // Create struct
    let vtab = Box::new(VectorLiteTab {
        base: mem::zeroed(),
        index: None,
        dimension,
        metric,
    });

    // Initialize index
    let max_elements = vtab::parse_max_elements(&args_bytes).unwrap_or(100_000);
    let ef_construction = vtab::parse_ef_construction(&args_bytes).unwrap_or(200);
    let m = vtab::parse_m(&args_bytes).unwrap_or(16);

    // Initialize HNSW
    // We modify the box before leaking
    let vtab_ptr = Box::into_raw(vtab);
    let vtab = &mut *vtab_ptr;

    match HnswIndex::new(dimension, max_elements, m, ef_construction, metric) {
        Ok(idx) => vtab.index = Some(idx),
        Err(_) => {
            let _ = Box::from_raw(vtab_ptr); // free
            return ffi::SQLITE_ERROR;
        }
    }

    *pp_vtab = &mut vtab.base;
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_create(
    db: *mut ffi::sqlite3,
    aux: *mut c_void,
    argc: c_int,
    argv: *const *const c_char,
    pp_vtab: *mut *mut ffi::sqlite3_vtab,
    pz_err: *mut *mut c_char,
) -> c_int {
    x_connect(db, aux, argc, argv, pp_vtab, pz_err)
}

unsafe extern "C" fn x_disconnect(vtab: *mut ffi::sqlite3_vtab) -> c_int {
    // Reconstruct Box and drop
    let vtab = Box::from_raw(vtab as *mut VectorLiteTab);
    drop(vtab);
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_destroy(vtab: *mut ffi::sqlite3_vtab) -> c_int {
    x_disconnect(vtab)
}

unsafe extern "C" fn x_best_index(
    _vtab: *mut ffi::sqlite3_vtab,
    info: *mut ffi::sqlite3_index_info,
) -> c_int {
    let info = &mut *info;

    let constraints = slice::from_raw_parts(info.aConstraint, info.nConstraint as usize);
    let usage = slice::from_raw_parts_mut(info.aConstraintUsage, info.nConstraint as usize);

    let mut has_match = false;
    for (i, c) in constraints.iter().enumerate() {
        // embedding is column 0
        if c.iColumn == 0 && c.op == (ffi::SQLITE_INDEX_CONSTRAINT_MATCH as u8) && c.usable != 0 {
            usage[i].argvIndex = 1;
            usage[i].omit = 1;
            has_match = true;
        }
    }

    if has_match {
        info.estimatedCost = 1000.0;
        info.estimatedRows = 100;
        info.idxNum = 1;
    } else {
        info.estimatedCost = 1_000_000.0;
        info.estimatedRows = 100_000;
        info.idxNum = 0;
    }

    ffi::SQLITE_OK
}

unsafe extern "C" fn x_open(
    _vtab: *mut ffi::sqlite3_vtab,
    pp_cursor: *mut *mut ffi::sqlite3_vtab_cursor,
) -> c_int {
    let cursor = Box::new(VectorLiteCursor {
        base: mem::zeroed(),
        results: Vec::new(),
        position: 0,
    });
    *pp_cursor = &mut (*Box::into_raw(cursor)).base;
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_close(cursor: *mut ffi::sqlite3_vtab_cursor) -> c_int {
    let cursor = Box::from_raw(cursor as *mut VectorLiteCursor);
    drop(cursor);
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_filter(
    cursor: *mut ffi::sqlite3_vtab_cursor,
    idx_num: c_int,
    _idx_str: *const c_char,
    argc: c_int,
    argv: *mut *mut ffi::sqlite3_value,
) -> c_int {
    let cursor = &mut *(cursor as *mut VectorLiteCursor);
    let vtab = &*(cursor.base.pVtab as *mut VectorLiteTab);

    cursor.results.clear();
    cursor.position = 0;

    if idx_num == 1 && argc >= 1 {
        // Search
        let args = slice::from_raw_parts(argv, argc as usize);
        let query_val = args[0];

        let type_code = ffi::sqlite3_value_type(query_val);

        let blob = ffi::sqlite3_value_blob(query_val);
        let text = ffi::sqlite3_value_text(query_val);

        let query_vec: Option<Vec<f32>> = if !blob.is_null() && type_code == ffi::SQLITE_BLOB {
            None
        } else if !text.is_null() {
            let s = CStr::from_ptr(text as *const c_char).to_string_lossy();
            serde_json::from_str(&s).ok()
        } else {
            None
        };

        if let Some(q) = query_vec {
            if let Some(index) = &vtab.index {
                if let Ok(results) = index.search(&q, 10, 100) {
                    cursor.results = results;
                }
            }
        }
    }
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_next(cursor: *mut ffi::sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(cursor as *mut VectorLiteCursor);
    cursor.position += 1;
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_eof(cursor: *mut ffi::sqlite3_vtab_cursor) -> c_int {
    let cursor = &mut *(cursor as *mut VectorLiteCursor);
    if cursor.position >= cursor.results.len() {
        1
    } else {
        0
    }
}

unsafe extern "C" fn x_column(
    cursor: *mut ffi::sqlite3_vtab_cursor,
    ctx: *mut ffi::sqlite3_context,
    i: c_int,
) -> c_int {
    let cursor = &mut *(cursor as *mut VectorLiteCursor);
    if cursor.position < cursor.results.len() {
        let (_rowid, distance) = cursor.results[cursor.position];
        match i {
            0 => ffi::sqlite3_result_null(ctx), // embedding
            1 => ffi::sqlite3_result_double(ctx, distance as f64), // distance
            _ => ffi::sqlite3_result_null(ctx),
        }
    }
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_rowid(
    cursor: *mut ffi::sqlite3_vtab_cursor,
    p_rowid: *mut ffi::sqlite3_int64,
) -> c_int {
    let cursor = &mut *(cursor as *mut VectorLiteCursor);
    if cursor.position < cursor.results.len() {
        *p_rowid = cursor.results[cursor.position].0;
    }
    ffi::SQLITE_OK
}

unsafe extern "C" fn x_update(
    vtab: *mut ffi::sqlite3_vtab,
    argc: c_int,
    argv: *mut *mut ffi::sqlite3_value,
    p_rowid: *mut ffi::sqlite3_int64,
) -> c_int {
    let vtab = &mut *(vtab as *mut VectorLiteTab);

    if argc == 1 {
        // DELETE
        return ffi::SQLITE_ERROR;
    }

    // INSERT/UPDATE
    // argv[0] = old rowid
    // argv[1] = new rowid (value)
    // argv[2] = list of columns... embedding is col 0 in table def (rowid is special)
    // So argv[2] is embedding.

    let args = slice::from_raw_parts(argv, argc as usize);
    let new_rowid_ptr = args[1];
    let embedding_ptr = args[2]; // column 0

    let rowid_type = ffi::sqlite3_value_type(new_rowid_ptr);
    let rowid = if rowid_type == ffi::SQLITE_INTEGER {
        ffi::sqlite3_value_int64(new_rowid_ptr)
    } else {
        return ffi::SQLITE_ERROR;
    };

    let embed_type = ffi::sqlite3_value_type(embedding_ptr);
    let vector: Option<Vec<f32>> = if embed_type == ffi::SQLITE_TEXT {
        let text = ffi::sqlite3_value_text(embedding_ptr);
        let s = CStr::from_ptr(text as *const c_char).to_string_lossy();
        serde_json::from_str(&s).ok()
    } else {
        None
    };

    if let Some(v) = vector {
        if let Some(index) = &vtab.index {
            match index.insert(rowid, v) {
                Ok(_) => {
                    *p_rowid = rowid;
                    ffi::SQLITE_OK
                }
                Err(_) => ffi::SQLITE_ERROR,
            }
        } else {
            ffi::SQLITE_ERROR
        }
    } else {
        ffi::SQLITE_ERROR
    }
}

use std::{mem, os::raw::c_int};

use sqlite_loadable::{
    api, define_virtual_table, prelude::*,
    table::{BestIndexError, IndexInfo, VTab, VTabArguments, VTabCursor},
    Result,
};

// 1. Define your table structure
#[repr(C)]
struct KubernetesTable {
    base: sqlite3_vtab,
}

impl<'vtab> VTab<'vtab> for KubernetesTable {
    type Aux = ();
    type Cursor = KubernetesCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        _args: VTabArguments,
    ) -> Result<(String, Self)> {
        // Define the SQL schema your virtual table exposes
        let schema = "CREATE TABLE x(id INTEGER, data TEXT);";
        let vtab = KubernetesTable {
            base: unsafe { mem::zeroed() },
        };
        Ok((schema.to_string(), vtab))
    }

    fn best_index(&self, _info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        Ok(())
    }

    fn open(&mut self) -> Result<Self::Cursor> {
        Ok(KubernetesCursor {
            base: unsafe { mem::zeroed() },
            row_id: 0,
        })
    }
}

// 2. Define how SQLite iterates through your table rows
#[repr(C)]
struct KubernetesCursor {
    base: sqlite3_vtab_cursor,
    row_id: i64,
}

impl VTabCursor for KubernetesCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        self.row_id = 1; // Reset iterator to start
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.row_id += 1; // Move to the next row
        Ok(())
    }

    fn eof(&self) -> bool {
        self.row_id > 5 // Stop after 5 rows for this dummy example
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        // Output data depending on requested column index `i`
        match i {
            0 => api::result_int64(context, self.row_id),
            1 => api::result_text(context, format!("Row number {}", self.row_id))?,
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.row_id)
    }
}

// 3. Register the extension initialization hook
#[sqlite_entrypoint]
fn sqlite3_extension_init(db: *mut sqlite3) -> Result<()> {
    define_virtual_table::<KubernetesTable>(db, "kubernetes_vtab", None)?;
    Ok(())
}

use std::os::raw::c_int;

use sqlite_loadable::{api, prelude::*, table::VTabCursor, Result};

pub struct Debug;

impl Debug {
    pub fn schema() -> String {
        format!("CREATE TABLE debug(id INTEGER, data TEXT);")
    }
}

#[repr(C)]
pub struct DebugCursor {
    pub base: sqlite3_vtab_cursor,
    pub row_id: i64,
}

impl VTabCursor for DebugCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        self.row_id = 1;
        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.row_id += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.row_id > 5
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match i {
            0 => api::result_int64(context, self.row_id),
            1 => api::result_text(context, format!("data-{}", self.row_id))?,
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.row_id)
    }
}

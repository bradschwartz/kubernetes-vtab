use std::os::raw::c_int;

use sqlite_loadable::{
    api,
    table::{VTabCursor},
    prelude::*,
    Result,
};

pub struct Pods;

impl Pods {
    pub fn schema() -> String {
        format!("CREATE TABLE pods(name TEXT, namespace TEXT, status TEXT, age TEXT, restart_count INTEGER, containers TEXT);")
    }
}

#[repr(C)]
pub struct PodsCursor {
    base: sqlite3_vtab_cursor,
    row_id: i64,
    restart_count: i64,
}

impl VTabCursor for PodsCursor {
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
            1 => api::result_text(context, format!("namespace-{}", self.row_id))?,
            2 => api::result_text(context, format!("status-{}", self.row_id))?,
            3 => api::result_text(context, format!("age-{}", self.row_id))?,
            4 => api::result_int64(context, self.restart_count),
            5 => api::result_text(context, format!("containers-{}", self.row_id))?,
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        Ok(self.row_id)
    }
}
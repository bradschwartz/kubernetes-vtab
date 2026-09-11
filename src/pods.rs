use std::os::raw::c_int;

use k8s_openapi::api::core::v1::Pod;
use kube::{
    api::{Api, ListParams},
    Client,
};
use sqlite_loadable::{api, prelude::*, table::VTabCursor, Result};

pub struct Pods;

impl Pods {
    pub fn schema() -> String {
        "CREATE TABLE pods(name TEXT, namespace TEXT, status TEXT, restart_count INTEGER);"
            .to_string()
    }
}

#[repr(C)]
pub struct PodsCursor {
    pub base: sqlite3_vtab_cursor,
    pub row_id: i64,
    pub pods: Vec<Pod>,
}

impl VTabCursor for PodsCursor {
    fn filter(
        &mut self,
        _idx_num: c_int,
        _idx_str: Option<&str>,
        _values: &[*mut sqlite3_value],
    ) -> Result<()> {
        self.row_id = 0;

        // this seems to get called a single time when the cursor is created
        // so we should do the fetching here
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let client = rt
            .block_on(async { Client::try_default().await })
            .map_err(|e| {
                sqlite_loadable::Error::new(sqlite_loadable::ErrorKind::Message(format!(
                    "Failed to create client: {}",
                    e
                )))
            })?;
        let api: Api<Pod> = Api::all(client);
        let pods = rt
            .block_on(async { api.list(&ListParams::default()).await })
            .map_err(|e| {
                sqlite_loadable::Error::new(sqlite_loadable::ErrorKind::Message(format!(
                    "Failed to list pods: {}",
                    e
                )))
            })?;
        self.pods = pods.items;

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.row_id += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.row_id == self.pods.len() as i64
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let current_pod = self.pods[self.row_id as usize].clone();
        match i {
            // pod name
            0 => api::result_text(context, current_pod.metadata.name.as_ref().unwrap())?,
            // namespace
            1 => api::result_text(
                context,
                current_pod.metadata.namespace.as_ref().unwrap().clone(),
            )?,
            // status
            2 => api::result_text(
                context,
                current_pod.status.as_ref().unwrap().phase.as_ref().unwrap(),
            )?,
            // restart count
            3 => api::result_int(
                context,
                current_pod
                    .status
                    .as_ref()
                    .unwrap()
                    .container_statuses
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|s| s.restart_count)
                    .sum::<i32>(),
            ),
            _ => (),
        }
        Ok(())
    }

    fn rowid(&self) -> Result<i64> {
        dbg!("rowid");
        Ok(self.row_id)
    }
}

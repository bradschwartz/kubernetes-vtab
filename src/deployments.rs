use std::os::raw::c_int;

use k8s_openapi::api::apps::v1::Deployment;
use kube::{
    api::{Api, ListParams},
    Client,
};
use sqlite_loadable::{api, prelude::*, table::VTabCursor, Result};

pub struct Deployments;

impl Deployments {
    pub fn schema() -> String {
        "CREATE TABLE deployments(name TEXT, namespace TEXT, ready_replicas INTEGER, replicas INTEGER);"
            .to_string()
    }
}

#[repr(C)]
pub struct DeploymentsCursor {
    pub base: sqlite3_vtab_cursor,
    pub row_id: i64,
    pub deployments: Vec<Deployment>,
}

impl VTabCursor for DeploymentsCursor {
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
        let api: Api<Deployment> = Api::all(client);
        let deployments = rt
            .block_on(async { api.list(&ListParams::default()).await })
            .map_err(|e| {
                sqlite_loadable::Error::new(sqlite_loadable::ErrorKind::Message(format!(
                    "Failed to list deployments: {}",
                    e
                )))
            })?;
        self.deployments = deployments.items;

        Ok(())
    }

    fn next(&mut self) -> Result<()> {
        self.row_id += 1;
        Ok(())
    }

    fn eof(&self) -> bool {
        self.row_id == self.deployments.len() as i64
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        let current_deployment = self.deployments[self.row_id as usize].clone();
        match i {
            // deployment name
            0 => api::result_text(context, current_deployment.metadata.name.as_ref().unwrap())?,
            // namespace
            1 => api::result_text(
                context,
                current_deployment
                    .metadata
                    .namespace
                    .as_ref()
                    .unwrap()
                    .clone(),
            )?,
            // ready replicas
            2 => api::result_int(
                context,
                current_deployment
                    .status
                    .as_ref()
                    .unwrap()
                    .ready_replicas
                    .unwrap_or_default(),
            ),
            // replicas
            3 => api::result_int(
                context,
                current_deployment
                    .status
                    .as_ref()
                    .unwrap()
                    .replicas
                    .unwrap_or_default(),
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

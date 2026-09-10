pub mod pods;
pub mod debug;

use std::{mem, os::raw::c_int};

use sqlite_loadable::{
    api, define_virtual_table,
    prelude::*,
    table::{BestIndexError, IndexInfo, VTab, VTabArguments, VTabCursor},
    vtab_argparse::{self, ConfigOption, ConfigOptionValue},
    Result,
};

// 1. Define your table structure
#[repr(C)]
struct KubernetesTable {
    base: sqlite3_vtab,
}

fn get_resource(arguments: &[vtab_argparse::Argument]) -> Result<&str> {
    let error = sqlite_loadable::Error::new(sqlite_loadable::ErrorKind::Message(
        "`resource` argument is required".to_string(),
    ));
    // go through arguments and find the one with the key "resource", returning the value
    // if none exist, return an error
    let resource = arguments
        .iter()
        .find(|arg| match arg {
            vtab_argparse::Argument::Config(config) => config.key == "resource",
            _ => false,
        })
        .map(|arg| match arg {
            vtab_argparse::Argument::Config(ConfigOption {
                value: ConfigOptionValue::Quoted(value),
                ..
            }) => value.as_str(),
            _ => unreachable!(),
        });
    resource.ok_or(error)
}

impl<'vtab> VTab<'vtab> for KubernetesTable {
    type Aux = ();
    type Cursor = KubernetesCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)> {
        // Define the SQL schema your virtual table exposes
        // dbg!(args.arguments);
        let arguments = args
            .arguments
            .iter()
            .map(|arg| vtab_argparse::parse_argument(arg).unwrap())
            .collect::<Vec<vtab_argparse::Argument>>();
        // dbg!(arguments);
        // require `resource` argument, as that's how we know if it's Pods/Deployments/etc
        let resource = get_resource(&arguments)?;
        let schema = match resource {
            "pods" => pods::Pods::schema(),
            "debug" => debug::Debug::schema(),
            _ => {
                return Err(sqlite_loadable::Error::new(
                    sqlite_loadable::ErrorKind::Message(format!("Unknown resource: {}", resource)),
                ))
            }
        };
        let vtab = KubernetesTable {
            base: unsafe { mem::zeroed() },
        };
        Ok((schema, vtab))
    }

    fn best_index(&self, _info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        Ok(())
    }

    fn open(&mut self) -> Result<Self::Cursor> {
        Ok(KubernetesCursor {
            base: unsafe { mem::zeroed() },
            row_id: 0,
            resource: "pods".to_string(),
        })
    }
}

// 2. Define how SQLite iterates through your table rows
// Using a single cursor that can handle different resources based on the `resource` field
#[repr(C)]
struct KubernetesCursor {
    base: sqlite3_vtab_cursor,
    row_id: i64,
    resource: String,
}

impl VTabCursor for KubernetesCursor {
    fn filter(
        &mut self,
        idx_num: c_int,
        idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
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
        // Output data depending on requested column index `i`
        // and the resource type stored in the cursor
        match i {
            0 => api::result_int64(context, self.row_id),
            1 => {
                if self.resource == "pods" {
                    api::result_text(context, format!("namespace-{}", self.row_id))?
                } else {
                    api::result_text(context, format!("data-{}", self.row_id))?
                }
            }
            2 => {
                if self.resource == "pods" {
                    api::result_text(context, format!("status-{}", self.row_id))?
                } else {
                    api::result_text(context, "")?
                }
            }
            3 => {
                if self.resource == "pods" {
                    api::result_text(context, format!("age-{}", self.row_id))?
                } else {
                    api::result_text(context, "")?
                }
            }
            4 => {
                api::result_int64(context, 0)
            }
            5 => {
                if self.resource == "pods" {
                    api::result_text(context, format!("containers-{}", self.row_id))?
                } else {
                    api::result_text(context, "")?
                }
            }
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

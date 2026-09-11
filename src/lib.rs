pub mod debug;
pub mod pods;

use std::{mem, os::raw::c_int};

use crate::debug::DebugCursor;
use crate::pods::PodsCursor;
use sqlite_loadable::{
    define_virtual_table,
    prelude::*,
    table::{BestIndexError, IndexInfo, VTab, VTabArguments, VTabCursor},
    vtab_argparse::{self, ConfigOption, ConfigOptionValue},
    Result,
};

// 1. Define your table structure
#[repr(C)]
struct KubernetesTable {
    base: sqlite3_vtab,
    resource: String,
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

// 2. Define the cursor enum that can be either Pods or Debug cursor
#[repr(C)]
enum KubernetesCursor {
    Pods {
        #[allow(dead_code)]
        base: sqlite3_vtab_cursor,
        pods_cursor: PodsCursor,
    },
    Debug {
        #[allow(dead_code)]
        base: sqlite3_vtab_cursor,
        debug_cursor: DebugCursor,
    },
}

impl VTabCursor for KubernetesCursor {
    fn filter(
        &mut self,
        idx_num: c_int,
        idx_str: Option<&str>,
        values: &[*mut sqlite3_value],
    ) -> Result<()> {
        match self {
            KubernetesCursor::Pods { pods_cursor, .. } => {
                pods_cursor.filter(idx_num, idx_str, values)
            }
            KubernetesCursor::Debug { debug_cursor, .. } => {
                debug_cursor.filter(idx_num, idx_str, values)
            }
        }
    }

    fn next(&mut self) -> Result<()> {
        match self {
            KubernetesCursor::Pods { pods_cursor, .. } => pods_cursor.next(),
            KubernetesCursor::Debug { debug_cursor, .. } => debug_cursor.next(),
        }
    }

    fn eof(&self) -> bool {
        match self {
            KubernetesCursor::Pods { pods_cursor, .. } => pods_cursor.eof(),
            KubernetesCursor::Debug { debug_cursor, .. } => debug_cursor.eof(),
        }
    }

    fn column(&self, context: *mut sqlite3_context, i: c_int) -> Result<()> {
        match self {
            KubernetesCursor::Pods { pods_cursor, .. } => pods_cursor.column(context, i),
            KubernetesCursor::Debug { debug_cursor, .. } => debug_cursor.column(context, i),
        }
    }

    fn rowid(&self) -> Result<i64> {
        match self {
            KubernetesCursor::Pods { pods_cursor, .. } => pods_cursor.rowid(),
            KubernetesCursor::Debug { debug_cursor, .. } => debug_cursor.rowid(),
        }
    }
}

// 3. Register the extension initialization hook
impl<'vtab> VTab<'vtab> for KubernetesTable {
    type Aux = ();
    type Cursor = KubernetesCursor;

    fn connect(
        _db: *mut sqlite3,
        _aux: Option<&Self::Aux>,
        args: VTabArguments,
    ) -> Result<(String, Self)> {
        // Define the SQL schema your virtual table exposes
        let arguments = args
            .arguments
            .iter()
            .map(|arg| vtab_argparse::parse_argument(arg).unwrap())
            .collect::<Vec<vtab_argparse::Argument>>();
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
            resource: resource.to_string(),
        };
        Ok((schema, vtab))
    }

    fn best_index(&self, _info: IndexInfo) -> core::result::Result<(), BestIndexError> {
        Ok(())
    }

    fn open(&mut self) -> Result<Self::Cursor> {
        match self.resource.as_str() {
            "pods" => Ok(KubernetesCursor::Pods {
                base: unsafe { mem::zeroed() },
                pods_cursor: PodsCursor {
                    base: unsafe { mem::zeroed() },
                    row_id: 0,
                    pods: vec![],
                },
            }),
            "debug" => Ok(KubernetesCursor::Debug {
                base: unsafe { mem::zeroed() },
                debug_cursor: DebugCursor {
                    base: unsafe { mem::zeroed() },
                    row_id: 0,
                },
            }),
            _ => unreachable!(),
        }
    }
}

// 4. Register the extension initialization hook
#[sqlite_entrypoint]
fn sqlite3_extension_init(db: *mut sqlite3) -> Result<()> {
    define_virtual_table::<KubernetesTable>(db, "kubernetes_vtab", None)?;
    Ok(())
}

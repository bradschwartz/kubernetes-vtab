pub struct Debug {
    id: i64,
    data: String,
}

impl Debug {
    pub fn schema() -> String {
        format!("CREATE TABLE debug(id INTEGER, data TEXT);")
    }
}
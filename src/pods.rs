// use k8s_openapi::api::core::v1::Container;

pub struct Pods {
    // name: String,
    // namespace: String,
    // status: String,
    // age: String,
    // restart_count: i32,
    // containers: Vec<Container>,
}

impl Pods {
    pub fn schema() -> String {
        format!("CREATE TABLE pods(name TEXT, namespace TEXT, status TEXT, age TEXT, restart_count INTEGER, containers TEXT);")
    }
}
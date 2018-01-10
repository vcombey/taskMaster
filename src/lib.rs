extern crate yaml_rust;
pub mod task_master {
    use std::collections::HashMap;
    pub mod process;
    pub mod service;

    Struct task_master {
        config_file: String,
        service_list: HashMap<String, service::Service>,
    }
}

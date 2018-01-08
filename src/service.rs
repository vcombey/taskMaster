use process::Config;
use std::collections::HashMap;
pub struct Service {
    name: String,
    process: HashMap<String, Config>,
}

impl Service {
    pub fn new(name: String) {
    }
}

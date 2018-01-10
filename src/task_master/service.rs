use process::Config;
use std::collections::HashMap;
struct Service {
    /// A service is a list of process bundled together. This allows
    /// us to call operations on a group of process rather than
    /// process 1 by 1.
    name: String,
    process_list: HashMap<String, Config>,
}

impl Service {
        /// Initialize a new service with name 'name' and no process.
    pub fn new(name: String) -> Service {
        Service {
            name,
            process_list: HashMap::new();
        }
    }

    pub fn get(&self, key: &str) -> Option<&HashMap<String, Process>> {
        	self.list.get(key)
    }
}

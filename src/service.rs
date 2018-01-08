use process::Process;
use std::collections::HashMap;
pub struct Service {
    /// A service is a list of process bundled together. This allows
    /// us to call operations on a group of process rather than
    /// process 1 by 1.
    list: Hashmap<String, HashMap<String, Process>>
}

impl Service {
    pub fn new(name: String) -> Service {
        /// Initialize a new service with name 'name' and no process.
        Service {
            list: HashMap::new().insert(name, HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Hashmap<String, Process>> {
        	self.list.get(key)
    }

    pub fn get_process(&self, key: &str) -> Option<&Process>
    pub fn push_process(&mut self, name: String, process: Process)
    {
        self.process.insert(name, process)
    }
}

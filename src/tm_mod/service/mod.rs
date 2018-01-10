use std::collections::HashMap;
use std::sync::mpsc;
use std::thread as std_thread;

// Loading submodule
pub mod thread;

// Loading
use super::config::Config;
use self::thread::process::Process;
use self::thread::Thread;

pub struct Service<'c> {
    pub name: String,
    thread_hash: HashMap<&'c str, Thread>,
}

impl<'c> Service<'c> {
    pub fn new(name: String) -> Service<'c> {
        Service {
            name,
            thread_hash: HashMap::new(),
        }
    }
    pub fn launch_from_hash(&mut self, map: HashMap<String, Config>) {
        for (name, config) in map.into_iter() {
            let (sender, receiver) = mpsc::channel();
            let clone_config = config.clone();
            let handle = std_thread::spawn(move || {
                let mut process = Process::new(clone_config, receiver);
                process.manage_program();
            });
            self.thread_hash.insert(&name, Thread::new(config, handle, sender));
        }
    }
}

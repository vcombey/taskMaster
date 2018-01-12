use std::collections::HashMap;
use std::sync::mpsc;
use std::thread as std_thread;

// Loading submodule
pub mod thread;

// Loading
use super::config::Config;
use self::thread::process::Process;
use self::thread::Thread;
//use tm_mod::cmd::Cmd;

pub struct Service {
    pub name: String,
    thread_hash: HashMap<String, Thread>,
}

impl Service {
    pub fn new(name: String) -> Service {
        Service {
            name,
            thread_hash: HashMap::new(),
        }
    }
    pub fn launch_from_hash(&mut self, map: HashMap<String, Config>, sender_to_main: &mut mpsc::Sender<String>) {
        for (name, config) in map.into_iter() {
            let (sender, receiver) = mpsc::channel();
            let clone_config = config.clone();
            let clone_sender_to_main = sender_to_main.clone();
            let handle = std_thread::spawn(move || {
                let mut process = Process::new(clone_config, receiver, clone_sender_to_main);
                process.manage_program();
            });
            self.thread_hash.insert(name.clone(), Thread::new(config, handle, sender));
        }
    }
}

use std::collections::HashMap;
use std::sync::mpsc;
use std::thread as std_thread;

// Loading submodule
pub mod thread;

// Loading
use super::config::Config;
use self::thread::process::Process;
use self::thread::Thread;
use tm_mod::cmd::Instruction;
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
    /*
    pub fn send_thread(&mut self, p_name: String, ins: Instruction) {
        for (thread_name, thread) in self.thread_hash.iter() {
            if &p_name == thread_name {
                //thread.send(ins);
            }
        }
    }
    */
    pub fn launch_from_hash(&mut self, map: HashMap<String, Config>, sender_to_main: &mut mpsc::Sender<String>) {
        for (name, config) in map.into_iter() {
            println!("name: {}", name);
            let mut handles = Vec::with_capacity(config.numprocs);
            let mut senders = Vec::with_capacity(config.numprocs);
            for _i in 0..config.numprocs {
                let (sender, receiver) = mpsc::channel();
                let clone_config = config.clone();
                let clone_sender_to_main = sender_to_main.clone();
                let handle = std_thread::spawn(move || {
                    let mut process = Process::new(clone_config, receiver, clone_sender_to_main);
                    process.manage_program();
                });
                handles.push(handle);
                senders.push(sender);
            }
            self.thread_hash.insert(name.clone(), Thread::new(config, handles, senders));
        }
    }
}

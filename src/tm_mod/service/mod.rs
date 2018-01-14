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

#[derive(Debug)]
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

    pub fn send_to_process(&self, p_name: &str, ins: Instruction) -> Result<(), String> {
        let thread = self.thread_hash.get(p_name)
            .ok_or(String::from("no process with that name"));

        thread.and_then(|t| t.send(ins))
    }

    pub fn send_to_all_process(&self, ins: Instruction) -> Result<(), String> {
        let mut res: Result<(), String> = Ok(());
        for (_, thread) in self.thread_hash.iter() {
            if let Some(e) = thread.send(ins).err() {
                res = match res {
                    Ok(_) => Err(e),
                    Err(o) => Err(format!("{}{}", o, e)),
                }
            }
        }
        res
    }

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

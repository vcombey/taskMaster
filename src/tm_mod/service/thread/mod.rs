pub mod process;
use super::super::config::Config;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use super::super::cmd::Instruction;

#[derive(Debug)]
pub struct Thread {
    config: Config,
    sender: Vec<Sender<Instruction>>,
    join_handle: Vec<JoinHandle<()>>,
}

impl Thread {
    pub fn new(config: Config, join_handle: Vec<JoinHandle<()>>, sender: Vec<Sender<Instruction>>) -> Thread {
        Thread {
            config,
            join_handle,
            sender,
        }
    }
    pub fn send(&self, ins: Instruction) -> Result<(), String> {
        let mut res: Result<(), String> = Ok(());
        for (i, sender) in self.sender.iter().enumerate() {
            let e = sender.send(ins).map_err(|_| format!("pb in sending to the thread no {}\n", i)).err();
            if let Some(e) = e {
                res = match res {
                    Ok(_) => Err(e),
                    Err(o) => Err(format!("{}{}", o, e)),
                }
            }
        }
        res
    }
}

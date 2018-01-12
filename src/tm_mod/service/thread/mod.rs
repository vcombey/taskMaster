pub mod process;
use super::super::config::Config;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use super::super::cmd::Instruction;

pub struct Thread {
    sender: Sender<Instruction>,
    config: Config,
    join_handle: JoinHandle<()>,
}

impl Thread {
    pub fn new(config: Config, join_handle: JoinHandle<()>, sender: Sender<Instruction>) -> Thread {
        Thread {
            config,
            join_handle,
            sender,
        }
    }
}

pub mod process;
use super::super::config::Config;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use super::super::cmd::Instruction;

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
}

pub mod process;
use super::super::config::Config;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use super::super::cmd::Cmd;

pub struct Thread {
    sender: Sender<Cmd>,
    config: Config,
    join_handle: JoinHandle<()>,
}

impl Thread {
    pub fn new(config: Config, join_handle: JoinHandle<()>, sender: Sender<Cmd>) -> Thread {
        Thread {
            config,
            join_handle,
            sender,
        }
    }
}

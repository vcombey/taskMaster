pub mod process;
use super::super::config::Config;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use super::super::cmd::Instruction;
use tm_mod::exec_error::ExecErrors;
use tm_mod::exec_error::ExecError;

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
    pub fn send(&self, ins: Instruction) -> Result<(), ExecErrors> {
        let e: Vec<ExecError> = self.sender.iter().enumerate().filter_map(|(i, s)| {
            s.send(ins).map_err(|_| ExecError::Sending((self.config.name.clone(), i))).err()
        })
        .collect();

        ExecErrors::result_from_e_vec(e)
    }
}

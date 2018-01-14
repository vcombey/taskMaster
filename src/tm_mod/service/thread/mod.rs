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
    join_handle: Option<Vec<JoinHandle<()>>>,
}

impl Thread {
    pub fn new(config: Config, join_handle: Vec<JoinHandle<()>>, sender: Vec<Sender<Instruction>>) -> Thread {
        Thread {
            config,
            join_handle: Some(join_handle),
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

impl Drop for Thread {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        self.send(Instruction::SHUTDOWN);

        println!("Shutting down all workers.");

        if let Some(join_handle) = self.join_handle.take() {
            for (i, j_h) in join_handle.into_iter().enumerate() {
                println!("Shutting down worker {}", i);
                j_h.join().unwrap();
            }
        }
    }
}

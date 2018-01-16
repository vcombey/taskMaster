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
    pub fn send(&self, thread_id: Option<usize>, ins: Instruction, nb_receive: &mut usize) -> Result<(), ExecErrors> {
        let e: Vec<ExecError> = match thread_id {
            Some(id) => match self.sender.get(id) {
                None => vec![ExecError::ThreadOutofRange((self.config.name.clone(), id))],
                Some(s) => s.send(ins).map_err(|_| ExecError::Sending((self.config.name.clone(), id))).and_then(|_| {*nb_receive+=1; Ok(())}).err().into_iter().collect(),
            },
            None => self.sender.iter().enumerate().filter_map(|(i, s)| {
                s.send(ins).map_err(|_| ExecError::Sending((self.config.name.clone(), i))).and_then(|_| {*nb_receive+=1; Ok(())}).err()
            }).collect(),
        };

        //*nb_receive += self.sender.len() - e.len();
        ExecErrors::result_from_e_vec(e)
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");

        let mut nb_receive = 0;
        self.send(None, Instruction::SHUTDOWN, &mut nb_receive);

        println!("Shutting down all workers.");

        if let Some(join_handle) = self.join_handle.take() {
            for (i, j_h) in join_handle.into_iter().enumerate() {
                println!("Shutting down worker {}", i);
                j_h.join().unwrap();
            }
        }
    }
}

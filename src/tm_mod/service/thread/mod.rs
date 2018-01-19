pub mod process;
use super::super::config::Config;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use super::super::cmd::Instruction;
use tm_mod::exec_error::ExecErrors;
use tm_mod::exec_error::ExecError;
use std::sync::mpsc;
use std::thread as std_thread;

use self::process::Process;

#[derive(Debug)]
pub struct Thread {
    join_handle: Option<JoinHandle<()>>,
    pub sender: Sender<(Instruction, Option<Config>)>,
}

impl Thread {
    pub fn new(config: Config, sender_to_main: mpsc::Sender<String>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let join_handle = std_thread::spawn(move || {
            let mut process = Process::new(config, receiver, sender_to_main);
            process.manage_program();
        });
        Thread {
            join_handle: Some(join_handle),
            sender,
        }
    }
}
    
impl Drop for Thread {
    fn drop(&mut self) {
        match self.sender.send((Instruction::SHUTDOWN, None)) {
            Err(_) => eprintln!("sending instruction shutdown failed"),
            _ =>{;},
        }
        if let Some(j_h) = self.join_handle.take() {
            match j_h.join() {
                Err(e) => eprintln!("{:?}", e),
                _ =>{;},
            }
        }
    }
}

#[derive(Debug)]
pub struct ThreadVec {
    pub config: Config,
    pub vec: Vec<Thread>,
}

impl ThreadVec {

    pub fn new(config: &Config, sender_to_main: &mut mpsc::Sender<String>) -> Self {
        let mut vec = Vec::with_capacity(config.numprocs);
        for _i in 0..config.numprocs {
            let thread = Thread::new(config.clone(), sender_to_main.clone());
            vec.push(thread);
        }
        ThreadVec {
            config: config.clone(),
            vec,
        }
    }

    pub fn send(&self, thread_id: Option<usize>, ins: Instruction, conf: Option<Config>, nb_receive: &mut usize) -> Result<(), ExecErrors> {
        let e: Vec<ExecError> = match thread_id {
            Some(id) => match self.vec.get(id) {
                None => vec![ExecError::ThreadOutofRange((self.config.name.clone(), id))],
                Some(t) => t.sender.send((ins, conf.clone())).map_err(|_| ExecError::Sending((self.config.name.clone(), id))).and_then(|_| {*nb_receive+=1; Ok(())}).err().into_iter().collect(),
            },
            None => self.vec.iter().enumerate().filter_map(|(i, t)| {
                t.sender.send((ins, conf.clone())).map_err(|_| ExecError::Sending((self.config.name.clone(), i))).and_then(|_| {*nb_receive+=1; Ok(())}).err()
            }).collect(),
        };
        ExecErrors::result_from_e_vec(e)
    }
}

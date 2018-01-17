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
use tm_mod::exec_error::ExecErrors;
use tm_mod::exec_error::ExecError;

//use tm_mod::cmd::Cmd;

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub thread_hash: HashMap<String, Thread>,
}

impl Service {
    pub fn new(name: String) -> Service {
        Service {
            name,
            thread_hash: HashMap::new(),
        }
    }

    pub fn contains_process(&self, p_name: &str) -> bool {
        self.thread_hash.contains_key(p_name)
    }

    pub fn send_to_process(&self, p_name: &str, thread_id: Option<usize>, ins: Instruction, nb_receive: &mut usize) -> Result<(), ExecErrors> {
        let thread = self.thread_hash.get(p_name)
            .ok_or(ExecError::ProcessName(String::from(p_name)));

        thread.map_err(|e| ExecErrors{e_vect: vec![e]})
            .and_then(|t| t.send(thread_id, ins, None, nb_receive))
    }

    pub fn send_to_all_process(&self, ins: Instruction, nb_receive: &mut usize) -> Result<(), ExecErrors>  {
        let e: Vec<ExecError> = self.thread_hash.values()
            .filter_map(|t| t.send(None, ins, None, nb_receive).err())
            .flat_map(|e| e.e_vect.into_iter())
            .collect();

        ExecErrors::result_from_e_vec(e)
    }

    fn launch_thread(&mut self, config: Config, sender_to_main: mpsc::Sender<String>) -> (std_thread::JoinHandle<()>, mpsc::Sender<(Instruction, Option<Config>)>) {
        let (sender, receiver) = mpsc::channel();
        let handle = std_thread::spawn(move || {
            let mut process = Process::new(config, receiver, sender_to_main);
            process.manage_program();
        });
        (handle, sender)
    }

    pub fn launch_from_hash(&mut self, process_hash: HashMap<String, Config>, sender_to_main: &mut mpsc::Sender<String>) {
        for (name, config) in process_hash.into_iter() {
            println!("name: {}", name);
            let mut handle_vec = Vec::with_capacity(config.numprocs);
            let mut sender_vec = Vec::with_capacity(config.numprocs);
            for _i in 0..config.numprocs {
                let (handle, sender) = self.launch_thread(config.clone(), sender_to_main.clone());
                handle_vec.push(handle);
                sender_vec.push(sender);
            }
            self.thread_hash.insert(name.clone(), Thread::new(config, handle_vec, sender_vec));
        }
    }

    pub fn reread(&mut self, reread_little_hash: &mut HashMap<String, Config>, sender_to_main: &mut mpsc::Sender<String>) {

        // Stop threads not found in the new hash but present in the old one
        self.thread_hash.retain( |thread_name, thread| { // If ret is false elem is removed from HashMap
            match reread_little_hash.get(thread_name) {
                None => { thread.send(None, Instruction::SHUTDOWN, None, &mut 0);
                          false },
                Some(_) => true,
            }
        });

        // Launch threads not found in the current thread_hash but present in the new one
        reread_little_hash.retain( |new_thread_name, config| { // If ret is false elem is removed from HashMap
            match self.thread_hash.get(new_thread_name) {
                None => {
                    let mut handle_vec = Vec::with_capacity(config.numprocs);
                    let mut sender_vec = Vec::with_capacity(config.numprocs);
                    for _i in 0..config.numprocs {
                        let (handle, sender) = self.launch_thread(config.clone(), sender_to_main.clone());
                        handle_vec.push(handle);
                        sender_vec.push(sender);
                    }
                    self.thread_hash.insert(new_thread_name.clone(), Thread::new(config.clone(), handle_vec, sender_vec));
                    false },
                Some(_) => true,
            }
        });
        // TREAT REMAINGS if fatal diff, kill + spawn new. If non fatal, update config => if numproc, spawn / despawn n threads with the name
    }
}

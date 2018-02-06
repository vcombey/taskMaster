use std::collections::HashMap;
use std::sync::mpsc;

// Loading submodule
pub mod thread;

// Loading
use super::config::Config;
use self::thread::ThreadVec;
use self::thread::Thread;
use tm_mod::cmd::Instruction;
use tm_mod::exec_error::ExecErrors;
use tm_mod::exec_error::ExecError;
use tm_mod::error_utils::print_err;

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub thread_hash: HashMap<String, ThreadVec>,
}

impl Service {
    pub fn new(name: String) -> Self {
        Service {
            name,
            thread_hash: HashMap::new(),
        }
    }

    pub fn contains_process(&self, p_name: &str) -> bool {
        self.thread_hash.contains_key(p_name)
    }

    pub fn send_to_process(
        &self,
        p_name: &str,
        thread_id: Option<usize>,
        ins: Instruction,
        nb_receive: &mut usize,
    ) -> Result<(), ExecErrors> {
        let thread = self.thread_hash
            .get(p_name)
            .ok_or(ExecError::ProcessName(String::from(p_name)));
        thread
            .map_err(|e| ExecErrors { e_vect: vec![e] })
            .and_then(|t| t.send(thread_id, ins, None, nb_receive))
    }

    pub fn send_to_all_process(
        &self,
        ins: Instruction,
        nb_receive: &mut usize,
    ) -> Result<(), ExecErrors> {
        let e: Vec<ExecError> = self.thread_hash
            .values()
            .filter_map(|t| t.send(None, ins, None, nb_receive).err())
            .flat_map(|e| e.e_vect.into_iter())
            .collect();

        ExecErrors::result_from_e_vec(e)
    }

    pub fn launch_from_hash(
        &mut self,
        process_hash: HashMap<String, Config>,
        sender_to_main: &mut mpsc::Sender<String>,
    ) {
        for (name, config) in process_hash.into_iter() {
            //eprintln!("name: {}", name);
            self.thread_hash
                .insert(name.clone(), ThreadVec::new(&config, sender_to_main));
        }
    }

    /// Function that compare an existing service to its new version defined in
    /// the config files.  Process not needed anymore are killed, the ones
    /// missing are launched, and the ones present in both are changed only if
    /// necessary, despawning them only if absolutely required
    pub fn reread(
        &mut self,
        reread_process_hash: &mut HashMap<String, Config>,
        sender_to_main: &mut mpsc::Sender<String>,
    ) {
        // Stop threads not found in the new hash but present in the old one
        // If ret is false elem is removed from HashMap
        self.thread_hash.retain(|thread_name, thread| {
            match reread_process_hash.get(thread_name) {
                None => false,
                Some(new_config) => match thread.config.fatal_cmp(&new_config) {
                    true => false,
                    false => {
                        // Two following conditions create/kill process
                        // depending on the diff between new numproc and old
                        // one.  Compares two configs. If a fatal difference is
                        // found, thread hosting the process with this config is
                        // killed. Otherwise, the new config is sent to the
                        // thread and its process who will now host it too.
                        // (even if the 2 configs are indentical). If numprocs
                        // differ, appropriate number of process must be killed
                        // added.
                        // the first condition is => for send REREAD to the
                        // threads even if the numproc hasn't changed
                        if new_config.numprocs >= thread.config.numprocs {
                            print_err(thread.send(
                                None,
                                Instruction::REREAD,
                                Some(new_config.clone()),
                                &mut 0,
                            ));
                            for _ in thread.config.numprocs..new_config.numprocs {
                                let t = Thread::new(new_config.clone(), sender_to_main.clone());
                                thread.vec.push(t);
                            }
                        } else if new_config.numprocs < thread.config.numprocs {
                            for _ in new_config.numprocs..thread.config.numprocs {
                                thread.vec.pop();
                            }
                            print_err(thread.send(
                                None,
                                Instruction::REREAD,
                                Some(new_config.clone()),
                                &mut 0,
                            ));
                        };
                        thread.config = new_config.clone();
                        true
                    }
                },
            }
        });

        // Launch threads not found in the current thread_hash but present in
        // the new one If ret is false elem is removed from HashMap
        reread_process_hash.retain(|new_thread_name, config| {
            match self.thread_hash.get(new_thread_name) {
                None => {
                    self.thread_hash.insert(
                        new_thread_name.clone(),
                        ThreadVec::new(&config, sender_to_main),
                    );
                    false
                }
                Some(_) => true,
            }
        });
    }
}

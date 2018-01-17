use std::collections::HashMap;
use std::sync::mpsc;

// Loading submodule
pub mod thread;

// Loading
use super::config::Config;
use self::thread::process::Process;
use self::thread::Thread_vec;
use self::thread::Thread;
use tm_mod::cmd::Instruction;
use tm_mod::exec_error::ExecErrors;
use tm_mod::exec_error::ExecError;

//use tm_mod::cmd::Cmd;

#[derive(Debug)]
pub struct Service {
    pub name: String,
    pub thread_hash: HashMap<String, Thread_vec>,
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

    pub fn launch_from_hash(&mut self, process_hash: HashMap<String, Config>, sender_to_main: &mut mpsc::Sender<String>) {
        for (name, config) in process_hash.into_iter() {
            //println!("name: {}", name);
            self.thread_hash.insert(name.clone(), Thread_vec::new(&config, sender_to_main));
        }
    }

    /// Function that compare an existing service to its new version defined in the config files.
    /// Process not needed anymore are killed, the ones missing are launched, and the ones present in both
    /// are changed only if necessary, despawning them only if absolutely required
    pub fn reread(&mut self, reread_process_hash: &mut HashMap<String, Config>, sender_to_main: &mut mpsc::Sender<String>) {

        // Stop threads not found in the new hash but present in the old one
        self.thread_hash.retain( |thread_name, thread| { // If ret is false elem is removed from HashMap
            match reread_process_hash.get(thread_name) {
                None => { /*thread.send(None, Instruction::SHUTDOWN, None, &mut 0);*/
                    false
                },
                Some(new_config) => match thread.config.fatal_cmp(&new_config) {
                        true => { /*thread.send(None, Instruction::SHUTDOWN, None, &mut 0) ;*/ false},
                        false => true,
                },
            }
        });

        // Launch threads not found in the current thread_hash but present in the new one
        reread_process_hash.retain( |new_thread_name, config| { // If ret is false elem is removed from HashMap
            match self.thread_hash.get(new_thread_name) {
                None => {
                    /*
                       let mut handle_vec = Vec::with_capacity(config.numprocs);
                       let mut sender_vec = Vec::with_capacity(config.numprocs);
                       for _i in 0..config.numprocs {
                       let (handle, sender) = Service::launch_thread(config.clone(), sender_to_main.clone());
                       handle_vec.push(handle);
                       sender_vec.push(sender);
                       }
                       self.thread_hash.insert(new_thread_name.clone(), Thread_vec::new(config.clone(), handle_vec, sender_vec));
                       */
                    self.thread_hash.insert(new_thread_name.clone(), Thread_vec::new(&config, sender_to_main));
                    false
                },
                Some(_) => true,
            }
        });

        // Compares two configs. If a fatal difference is found,
        // thread hosting the process with this config is
        // killed. Otherwise, the new config is sent to the thread and
        // its process who will now host it too. (even if the 2
        // configs are indentical). If numprocs differ, appropriate
        // number of process must be killed / added.
        for (process_name, new_config) in reread_process_hash {
            let thread = self.thread_hash.get_mut(process_name).unwrap();
            match thread.config.fatal_cmp(&new_config) {
                true => {;},

                false => { // Update configs. create numproc procs
                    println!("{:?}", thread);

                    // Two following conditions create/kill
                    // process depending on the diff between new
                    // numproc and old one.
                    println!("new , old: {} {}", new_config.numprocs, thread.config.numprocs );
                    if new_config.numprocs > thread.config.numprocs {
                        thread.send(None, Instruction::REREAD, Some(new_config.clone()), &mut 0);
                        for i in thread.config.numprocs..new_config.numprocs {
                            let t = Thread::new(new_config.clone(), sender_to_main.clone());
                            println!("new > old i: {}", i);
                            thread.vec.push(t);
                        };
                    }
                    else if new_config.numprocs < thread.config.numprocs {
                        for i in new_config.numprocs..thread.config.numprocs {
                            thread.vec.pop();
                            println!("new < old i: {}", i);
                        }
                        thread.send(None, Instruction::REREAD, Some(new_config.clone()), &mut 0);
                    };
                    thread.config = new_config.clone();
                }
            }
        }
    }
}

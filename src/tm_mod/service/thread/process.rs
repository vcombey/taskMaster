use std::process::Command;
use std::fs::File;
use std::process::Child;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Instant;
use std::os::unix::process::CommandExt;
extern crate nix;
use self::nix::sys::stat::umask;
use self::nix::sys::stat::Mode;

use std::io::Error;

#[derive(Debug,PartialEq)]
enum State {
    RUNNING,
    BACKOFF,
    STOPPED,
    UNLAUNCHED,
}
use tm_mod::config::Config;
use tm_mod::cmd::Instruction;

#[derive(Debug)]
pub struct Process {
    command: Command,
    config: Config,
    sender: Sender<String>,
    receiver: Receiver<Instruction>,
    child: Option<Child>,
    state: State,
}

impl Process {
    pub fn new(config: Config, receiver: Receiver<Instruction>, sender: Sender<String>) -> Process {
        Process {
            command: Command::new(config.argv.split(" ").next().unwrap()),
            config,
            receiver,
            sender,
            child: None,
            state: State::UNLAUNCHED,
        }
    }
    fn add_workingdir(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.workingdir {
            self.command.current_dir(string);
        }
        self
    }

    fn add_args(&mut self) -> &mut Process {
        if self.config.argv.len() > 1 {
            let args: Vec<&str> = self.config.argv.split(" ").collect();
            self.command.args(&args[1..]);
        }
        self
    }

    fn add_stdout(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.stdout {
            match File::open(string) {
                Ok(file) => {self.command.stdout(file);},
                Err(e) => eprintln!("{}", e),
            }
        }
        self
    }

    fn add_stderr(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.stderr {
            match File::open(string) {
                Ok(file) => {self.command.stderr(file);},
                Err(e) => eprintln!("{}", e),
            }
        }
        self
    }

    fn add_env(&mut self) -> &mut Process {
        if let Some(ref vect) = self.config.env {
            let v: Vec<(String, String)> = vect.to_vec();
            self.command.envs(v);
        }
        self
    }

    fn add_umask(&mut self) -> &mut Process {
        let conf_umask = self.config.umask;

        let call_umask = move || -> Result<(), Error> {
            let mode = Mode::from_bits(conf_umask).unwrap();
            umask(mode);
            Ok(())
        };

        self.command.before_exec(call_umask);
        self
    }

    fn spawn(&mut self) -> &mut Process {
        let child = self.command.spawn();
        if let Ok(child) = child {
            self.child = Some(child);
        }
        self
    }

    fn start(&mut self) -> &mut Process {
        self.add_args()
            .add_workingdir()
            .add_env()
            .add_stdout()
            .add_stderr()
            .add_umask()
            .spawn()
    }
    fn try_wait(&mut self) -> State{ 
        if let Some(ref mut child) = self.child {
            match child.try_wait() {

                /* le program has ended */
                Ok(Some(exit_status)) => {
                    match exit_status.code() {
                        Some(exit_status_code) => {
                            eprintln!("INFO exited: '{}' (exit status {}; expected)", 
                                      self.config.name, 
                                      exit_status_code);
                        }
                        None => {
                            eprintln!("INFO stopped: '{}' (terminated by SIGKILL) ", self.config.name);
                        }
                    }
                    return State::BACKOFF;
                },
                _ => { return State::RUNNING;}
            }
        }
        State::RUNNING
    }
    /// try launch the programe one time
    /// return after starttime if the program is still running
    /// or return if the program has exited before starttime
    fn try_launch(&mut self) -> bool {
        let mut success = false;
        self.start();
        if let Some(ref mut child) = self.child {
            let now = Instant::now();
            loop {
                match child.try_wait() {

                    /* le program has ended */
                    Ok(Some(exit_status)) => {
                        eprintln!("INFO spawned: '{}' with pid {:?}", self.config.name, child.id());
                        //eprintln!("duree: {:?}", duree);
                        let exit_status_code = exit_status.code().unwrap();
                        let nownow = Instant::now();
                        let duree = nownow.duration_since(now);

                        /* it is an unexpected ended */
                        if duree < self.config.starttime || !self.config.exitcodes.contains(&(exit_status_code as i64)) {
                            eprintln!("INFO exited: '{}' (exit status {}; not expected)", 
                                      self.config.name, 
                                      exit_status_code);

                            /* it is an expected ended */
                        } else {
                            success = true;
                            eprintln!("INFO exited: '{}' (exit status {}; expected)", 
                                      self.config.name, 
                                      exit_status_code);
                        }
                        break ;
                    },
                    /* le program has not ended yet : check the time*/
                    Ok(None) => {
                        let nownow = Instant::now();
                        let duree = nownow.duration_since(now);
                        if duree > self.config.starttime {
                            eprintln!("INFO spawned: '{}' with pid {:?}", self.config.name, child.id());
                            success = true;
                            break ;
                        } else { 
                            continue ;
                        }
                    }
                    Err(e) => eprintln!("error attempting to wait: {}", e),
                }
            }
        }
        success
    }
    /// call in loop try launch no more than startretries or
    /// until the program has started
    fn try_execute(&mut self) -> State {
        for nb_try in 0..self.config.startretries+1{
            println!("nb_try {}, startretries {}", nb_try, self.config.startretries);
            if self.try_launch() {
                return State::RUNNING;
            }
        }
        State::BACKOFF
    }
    fn stop(&mut self) {
        if let Some(ref mut child) = self.child {
            match child.kill() {
                Ok(_) => {;}, 
                Err(_) => eprintln!("{}: ERROR (not running)", self.config.name),
            }
        }
    }
    fn status(&mut self) {
        self.sender.send(format!("{}: {:?}", self.config.name, self.state));
    }
    fn handle_cmd(&mut self, cmd: Instruction) {
        match cmd {
            Instruction::STOP => {
                self.stop();
            },
            Instruction::STATUS => { ;
                self.status();
            },
            Instruction::START => { ;
            },
            Instruction::RESTART => { ;
            },
            Instruction::RELOAD => { ;
            },
            Instruction::SHUTDOWN => { ;
            },
        }
    }
    pub fn manage_program(&mut self) {
        self.state = self.try_execute();
        loop {
            match self.receiver.try_recv() {
                Ok(ins) => {
                    eprintln!("INFO process '{}' receive {:?}", self.config.name, ins);
                    if ins == Instruction::SHUTDOWN {
                        break ;
                    }
                    self.handle_cmd(ins);
                },
                Err(_) => { ; },
            }
            if self.try_wait() == State::BACKOFF{
                self.child = None;
            }
        }
    }
}

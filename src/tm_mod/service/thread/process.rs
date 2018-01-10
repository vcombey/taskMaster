use std::process::Command;
use std::fs::File;
use std::process::Child;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use super::super::super::config::Config;
use super::super::super::cmd::Cmd;

#[derive(Debug)]
pub struct Process {
    command: Command,
    config: Config,
    receiver: Receiver<Cmd>,
    child: Option<Child>,
}

impl Process {
    pub fn new(config: Config, receiver: Receiver<Cmd>) -> Process {
        Process {
            command: Command::new(config.argv.split(" ").next().unwrap()),
            config,
            receiver,
            child: None,
        }
    }
    fn add_workingdir(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.workingdir {
            self.command.env("PWD", string);
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
                Err(e) => eprintln!("error{:?}", e),
            }
        }
        self
    }

    fn add_stderr(&mut self) -> &mut Process {
        if let Some(ref string) = self.config.stderr {
            match File::open(string) {
                Ok(file) => {self.command.stderr(file);},
                Err(e) => eprintln!("error{:?}", e),
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

    pub fn spawn(&mut self) -> &mut Process {
        let child = self.command.spawn();
        if let Ok(child) = child {
            self.child = Some(child);
        }
        self
    }

    pub fn start(&mut self) -> &mut Process {
        self.add_args()
            .add_workingdir()
            .add_env()
            .add_stdout()
            .add_stderr()
            .spawn()
    }
    
    /// try launch the programe one time
    /// return after starttime if the program is still running
    /// or return if the program has exited before starttime
    pub fn try_launch(&mut self) -> bool {
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
    pub fn try_execute(&mut self) -> bool {
        for nb_try in 0..self.config.startretries+1{
        println!("nb_try {}, startretries {}", nb_try, self.config.startretries);
            if self.try_launch() {
                return true ;
           }
        }
        false
    }
    pub fn manage_program(&mut self) {
        self.try_execute();
        loop {
            match self.receiver.try_recv() {
                Ok(cmd) => { 
                    eprintln!("INFO process '{}' receive {:?}", self.config.name, cmd);
                },
                Err(e) => continue ,
            }
        }
    }
}

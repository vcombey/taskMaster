use std::time::{Duration, Instant};
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};

#[derive(Debug,Clone)]
enum Autorestart {
    TRUE,
    FALSE,
    UNEXPECTED,
}

#[derive(Debug,Clone)]
pub struct Config {
    /// The Config struct represents all the informations we want
    /// to have about a single process we are supervising.
    name: String,
    argv: String,
    workingdir: Option<String>,
    autostart: bool,
    env: Option<Vec<(String, String)>>,
    stdout: Option<String>,
    stderr: Option<String>,
    exitcodes: Vec<i64>,
    startretries: u64,
    umask: i64,
    autorestart: Autorestart,
    starttime: Duration,
    stopsignal: i64,
    stoptime: Duration,
    numprocs: i64,
}


impl Config {
    pub fn new(name: String,
               argv: String, 
               workingdir: Option<&str>,
               autostart: Option<bool>,
               env: Option<Vec<(String, String)>>,
               stdout: Option<&str>,
               stderr: Option<&str>,
               exitcodes: Option<Vec<i64>>,
               startretries: Option<i64>,
               umask: Option<i64>,
               autorestart: Option<&str>,
               starttime: Option<i64>,
               stopsignal: Option<i64>,
               stoptime: Option<i64>,
               numprocs: Option<i64>
              ) -> Config {
        /// Function to generate a new instance of a Config strct.
        /// Only mandatory arguments are name and command.
        /// Other arguments can be skipped by giving `None' 
        Config {
            name, 
            argv,
            workingdir: match workingdir {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            autostart: match autostart {
                Some(value) => value,
                None => true,
            },
            env,
            stdout: match stdout {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            stderr: match stderr {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            exitcodes: match exitcodes {
                Some(v) => v,
                None => vec![1, 2],
            },
            startretries: match startretries {
                Some(i) => i as u64, // TODO check coherence of types i64 and u64
                None => 3,
            },
            umask: match umask {
                Some(i) => i,
                None => 0700,
            },
            autorestart: match autorestart { //TODO: voir ce que c'est
                Some(slice) => if slice == "unexpected" { Autorestart::UNEXPECTED } 
                else if slice == "true" { Autorestart::TRUE }
                else if slice == "false"{ Autorestart::FALSE }
                else { panic!("bad value for autorestart") }
                None => Autorestart::UNEXPECTED,
            },
            starttime:  match starttime {
                Some(i) => Duration::from_secs(i as u64),
                None => Duration::from_secs(1),
            },
            stopsignal: match stopsignal {
                Some(i) => i,
                None => 0, //TODO: mettre TERM,
            },
            stoptime:  match stoptime {
                Some(i) => Duration::from_secs(i as u64),
                None => Duration::from_secs(10),
            },
            numprocs:  match numprocs {
                Some(i) => i,
                None => 1,
            },
        }
    }
    pub fn from_yaml(name: &str, argv:&str, config: &Yaml) -> Config {
        /// Creates a Config instance from the process name and a
        /// Yaml struct representing the config options. Parses
        /// YAML into variables and calls new.

        // env is represented by a nested YAML into the current
        // config. Parsing it as a tuple of key, value.
        let env: Option<Vec<(String, String)>> = match (&config["env"]).as_hash() {
            Some(hash) => { Some(hash.iter()
                                 .map(|(name, argv)| {
                                     (String::from(name.as_str().unwrap()), 
                                      String::from(argv.as_str().unwrap()))
                                 }) //TODO: gerer les nombre
                                 .collect())
            },
            None => None,
        };

        // Exitcodes can be either one field, or many.
        let exitcodes =  match (&config["exitcodes"]).as_vec() {
            Some(v) => Some(v.iter().map(|a| {
                a.as_i64().unwrap()})
                            .collect()),
            None => match (&config["exitcodes"]).as_i64() {
                Some(i) => Some(vec![i]),
                None => None,
            },
        };
        Config::new(String::from(name),
        String::from(argv),
        (&config["workingdir"]).as_str(),
        (&config["autostart"]).as_bool(),
        env,
        (&config["stdout"]).as_str(),
        (&config["stderr"]).as_str(),
        exitcodes,
        (&config["startretries"]).as_i64(),
        (&config["umask"]).as_i64(),
        (&config["autorestart"]).as_str(),
        (&config["starttime"]).as_i64(),
        (&config["stopsignal"]).as_i64(),
        (&config["stoptime"]).as_i64(),
        (&config["numprocs"]).as_i64(),
        )
    }
}

use std::process::Command;
use std::fs::File;
use std::process::Child;
use std::sync::mpsc::Receiver;
use cmd;

#[derive(Debug)]
pub struct Process {
    command: Command,
    config: Config,
    receiver: Receiver<cmd::Cmd>,
    child: Option<Child>,
}

impl Process {
    pub fn new(config: Config, receiver: Receiver<cmd::Cmd>) -> Process {
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

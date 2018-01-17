// Imorting yaml
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};

// Imorting std
#[allow(unused_imports)]
use std::time::{Duration, Instant};
use nix::sys::signal::Signal::*;
use nix::sys::signal::Signal;

#[derive(Debug,Clone)]
pub enum Autorestart {
    TRUE,
    FALSE,
    UNEXPECTED,
}


#[derive(Debug,Clone)]
pub struct Config {
    /// The Config struct represents all the informations we want
    /// to have about a single process we are supervising.
    pub name: String,
    pub argv: String,
    pub workingdir: Option<String>,
    pub autostart: bool,
    pub env: Option<Vec<(String, String)>>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub exitcodes: Vec<i64>,
    pub startretries: u64,
    pub umask: u16,
    pub autorestart: Autorestart,
    pub starttime: Duration,
    pub stopsignal: Signal,
    pub stoptime: Duration,
    pub numprocs: usize,
}


impl Config {
    /// Function to generate a new instance of a Config strct.
    /// Only mandatory arguments are name and command.
    /// Other arguments can be skipped by giving `None' 
    pub fn new(name: &str,
               argv: &str, 
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
               stopsignal: Signal,
               stoptime: Option<i64>,
               numprocs: Option<i64>
              ) -> Self {
        Config {
            name:  String::from(name),
            argv:  String::from(argv),
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
                Some(i) => i as u16,
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
            stopsignal,
            stoptime:  match stoptime {
                Some(i) => Duration::from_secs(i as u64),
                None => Duration::from_secs(10),
            },
            numprocs:  match numprocs {
                Some(i) => {assert!(i > 0); i as usize},
                None => 1,
            },
        }
    }

    fn parse_signal(sig_name: &str) -> Option<Signal> {
        match sig_name {
            "HUP"=>   Some(SIGHUP),
            "INT"=>   Some(SIGINT),
            "QUIT"=>  Some(SIGQUIT),
            "ILL"=>   Some(SIGILL),
            "TRAP"=>  Some(SIGTRAP),
            "ABRT"=>  Some(SIGABRT),
            "BUS"=>   Some(SIGBUS),
            "FPE"=>   Some(SIGFPE),
            "KILL"=>  Some(SIGKILL),
            "USR1"=>  Some(SIGUSR1),
            "SEGV"=>  Some(SIGSEGV),
            "USR2"=>  Some(SIGUSR2),
            "PIPE"=>  Some(SIGPIPE),
            "ALRM"=>  Some(SIGALRM),
            "TERM"=>  Some(SIGTERM),
            "CHLD"=>  Some(SIGCHLD),
            "CONT"=>  Some(SIGCONT),
            "STOP"=>  Some(SIGSTOP),
            "TSTP"=>  Some(SIGTSTP),
            "TTIN"=>  Some(SIGTTIN),
            "TTOU"=>  Some(SIGTTOU),
            "URG"=>   Some(SIGURG),
            "XCPU"=>  Some(SIGXCPU),
            "XFSZ"=>  Some(SIGXFSZ),
            "VTALRM"=>Some(SIGVTALRM),
            "PROF"=>  Some(SIGPROF),
            "WINCH"=> Some(SIGWINCH),
            "IO"=>    Some(SIGIO),
            "SYS"=>   Some(SIGSYS),
            "EMT"=>   Some(SIGEMT),
            "INFO"=>  Some(SIGINFO),
            _ =>      None,
        }
    }

    /// Creates a Config instance from the process name and a
    /// Yaml struct representing the config options. Parses
    /// YAML into variables and calls new.
    pub fn from_yaml(name: &str, argv: &str, config: &Yaml) -> Config {

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

        let stop_signal = match (&config["stopsignal"]).as_str() {
            Some(slice) => self::Config::parse_signal(slice).unwrap(),
            None => SIGTERM,
        };
        Config::new(name,
                    argv,
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
                    stop_signal,
                    (&config["stoptime"]).as_i64(),
                    (&config["numprocs"]).as_i64(),
                    )
    }
}


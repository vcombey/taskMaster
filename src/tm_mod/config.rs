// Imorting yaml
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};

// Imorting std
#[allow(unused_imports)]
use std::time::{Duration, Instant};

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
    pub umask: i64,
    pub autorestart: Autorestart,
    pub starttime: Duration,
    pub stopsignal: i64,
    pub stoptime: Duration,
    pub numprocs: i64,
}


impl Config {
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
               stopsignal: Option<i64>,
               stoptime: Option<i64>,
               numprocs: Option<i64>
              ) -> Config {
        /// Function to generate a new instance of a Config strct.
        /// Only mandatory arguments are name and command.
        /// Other arguments can be skipped by giving `None' 
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
    pub fn from_yaml(name: &str, argv: &str, config: &Yaml) -> Config {
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
        (&config["stopsignal"]).as_i64(),
        (&config["stoptime"]).as_i64(),
        (&config["numprocs"]).as_i64(),
        )
    }
}

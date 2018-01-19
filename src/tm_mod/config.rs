
use yaml_rust::Yaml;
use yaml_rust::yaml;

use std::time::Duration;
use nix::sys::signal::Signal::*;
use nix::sys::signal::Signal;

#[derive(Debug,Clone)]
pub enum Autorestart {
    TRUE,
    FALSE,
    UNEXPECTED,
}

macro_rules! define_to_ref (
    ($name:ident, $t:ty, $yt:ident) => (
        pub fn $name<'a,'b>(yaml: &'a Yaml, champ: &'b str) -> Option<$t> {
            let yaml = &yaml[champ];
            if yaml.is_badvalue() {
                return None;
            }
            match *yaml {
                Yaml::$yt(ref v) => Some(v),
                _ => panic!("invalid type for: {}", champ),
            }
        }
    );
);

macro_rules! define_to (
    ($name:ident, $t:ident, $yt:ident) => (
        pub fn $name(yaml: &Yaml, champ: &str) -> Option<$t> {
            let yaml = &yaml[champ];
            if yaml.is_badvalue() {
                return None;
            }
            match *yaml {
                Yaml::$yt(v) => Some(v),
                _ => panic!("invalid type:"),
            }
        }
    );
);

define_to_ref!(to_str, &'a str, String);
define_to_ref!(to_hash, &'a yaml::Hash, Hash);
define_to!(to_i64, i64, Integer);
define_to!(to_bool, bool, Boolean);

#[derive(Debug, Clone)]
/// The Config struct represents all the informations we want
/// to have about a single process we are supervising.
pub struct Config {
    pub name: String, // fatal
    pub argv: String, // fatal
    pub workingdir: Option<String>, // fatal
    pub autostart: bool, // non-fatal
    pub env: Option<Vec<(String, String)>>, // fatal
    pub stdout: Option<String>, // fatal
    pub stderr: Option<String>, // fatal
    pub exitcodes: Vec<i64>, // non-fatal
    pub startretries: u64, // non-fatal
    pub umask: u16, // fatal
    pub autorestart: Autorestart, // non-fatal
    pub starttime: Duration, // non-fatal
    pub stopsignal: Signal, // non-fatal
    pub stoptime: Duration, // non-fatal
    pub numprocs: usize, // non-fatal
}

static VALID_FIELDS: [&'static str; 14] = ["cmd", "workingdir", "autostart", "env", "stdout", "stderr", "exitcodes", "startretries", "umask", "autorestart", "starttime", "stopsignal", "stoptime", "numprocs"];

impl Config {
    /// Creates a Config instance from the process name and a
    /// Yaml struct representing the config options. Parses
    /// YAML into variables and calls new.
    pub fn new(name: &str, argv: &str, config: &Yaml) -> Self {

        // env is represented by a nested YAML into the current
        // config. Parsing it as a tuple of key, value.
        let env: Option<Vec<(String, String)>> = match to_hash(config, "env") {
            Some(hash) => {
                Some(hash.iter()
                     .map(|(var, value)| {
                         (String::from((var).as_str().expect("Bad env keyword")), 
                          String::from((value).as_str().expect("Bad env value")))
                     }) //TODO: gerer les nombre
                     .collect())
            },
            None => None,
        };

        // Exitcodes can be either one field, or many.
        let exitcodes =  match (&config["exitcodes"]).as_vec() {
            Some(v) => Some(v.iter().map(|a| {
                a.as_i64().expect("Expected numeric value for field exitcode")})
                            .collect()),
            None => match to_i64(config, "exitcodes") {
                Some(i) => Some(vec![i]),
                None => None,
            },
        };

        let stopsignal = match to_str(config, "stopsignal") {
            Some(slice) => self::Config::parse_signal(slice).expect("Expected signal for field signal"),
            None => SIGTERM,
        };
        for (key, _) in config.as_hash().expect("Bad YAML format").iter() {
            if !VALID_FIELDS.contains(&key.as_str().expect("Expected str for field name")) {
                panic!("bad field: {}", key.as_str().unwrap());
            }
        }

        Config {
            name:  String::from(name),
            argv:  String::from(argv),
            workingdir: match to_str(config,"workingdir") {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            autostart: match to_bool(config,"autostart") {
                Some(value) => value,
                None => true,
            },
            env,
            stdout: match to_str(config, "stdout") {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            stderr: match to_str(config, "stderr") {
                Some(slice) => Some(String::from(slice)),
                None => None,
            },
            exitcodes: match exitcodes {
                Some(v) => v,
                None => vec![0],
            },
            startretries: match to_i64(config, "startretries") {
                Some(i) if i < 0 => panic!("startretries can't be negative"),
                Some(i) => i as u64,
                None => 3,
            },
            umask: match to_i64(config, "umask") {
                Some(i) if i < 0 => panic!("umask can't be negative"),
                Some(i) => i as u16,
                None => 0700,
            },
            autorestart: match to_str(config, "autorestart") {
                Some(slice) => match slice {
                    "unexpected" => Autorestart::UNEXPECTED,
                    "yes"=> Autorestart::TRUE,
                    "no" => Autorestart::FALSE,
                    _ => panic!("bad value for autorestart"),
                }
                None => Autorestart::UNEXPECTED,
            },
            starttime:  match to_i64(config, "starttime") {
                Some(i) if i < 0 => panic!("starttime can't be negative"),
                Some(i) => Duration::from_secs(i as u64),
                None => Duration::from_secs(1),
            },
            stopsignal,
            stoptime:  match to_i64(config, "stoptime") {
                Some(i) if i < 0 => panic!("stoptime can't be negative"),
                Some(i) => Duration::from_secs(i as u64),
                None => Duration::from_secs(10),
            },
            numprocs:  match to_i64(config, "numprocs") {
                Some(i) if i < 0 => panic!("numprocs can't be negative"),
                Some(i) => {assert!(i > 0); i as usize},
                None => 1,
            },
        }
    }

    /// Returns true if the 2 config have fatal differences (one that necessites
    /// restarting to apply)
    pub fn fatal_cmp(&self, other: &Config) -> bool {
        if self.name != other.name ||
            self.argv != other.argv ||
            self.workingdir != other.workingdir ||
            self.env != other.env ||
            self.stdout != other.stdout ||
            self.stderr != other.stderr ||
            self.umask != other.umask {
                return true;
            }
        false
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
}

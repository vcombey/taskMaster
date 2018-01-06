#[allow(unused_imports)]
use std::io;
#[allow(unused_imports)]
use std::io::Write;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs::File;
#[allow(unused_imports)]
use std::io::prelude::*;
extern crate yaml_rust;
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};
#[allow(unused_imports)]
use std::process::Command;
use std::time::{Duration, Instant};
use std::process::Child;

fn parse_argv (args: &[String]) -> (&str, &str)
{
    if args.len() < 3 {
        panic!("not enough arguments");
    }
    let option = &args[1];
    if option != "-c" {
        panic!("unknown option");
    }
    let filename = &args[2];

    (option, filename)
}

#[derive(Debug)]
struct Process {
    name: String,
    command: Command,
    argv: String,
    workingdir: Option<String>,
    autostart: bool,
    env: Option<Vec<(String, String)>>,
    stdout: Option<String>,
    stderr: Option<String>,
    exitcodes: Vec<i64>,
    startretries: i64,
    umask: i64,
    autorestart: i64,
    starttime: Duration,
    stopsignal: i64,
    stoptime: Duration,
    numprocs: i64,
}

impl Process {
    fn new(name: String,
           argv: String, 
           workingdir: Option<&str>,
           autostart: Option<bool>,
           env: Option<Vec<(String, String)>>,
           stdout: Option<&str>,
           stderr: Option<&str>,
           exitcodes: Option<Vec<i64>>,
           startretries: Option<i64>,
           umask: Option<i64>,
           autorestart: Option<i64>,
           starttime: Option<i64>,
           stopsignal: Option<i64>,
           stoptime: Option<i64>,
           numprocs: Option<i64>
          ) -> Process {
        Process {
            name, 
            command: Command::new(argv.split(" ").next().unwrap()),
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
                Some(i) => i,
                None => 3,
            },
            umask: match umask {
                Some(i) => i,
                None => 0700,
            },
            autorestart: match autorestart { //TODO: voir ce que c'est
                Some(i) => i,
                None => 0,
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
    fn add_workingdir(mut self) -> Self {
        if let Some(ref string) = self.workingdir {
            self.command.env("PWD", string);
        }
        self
    }
    fn add_args(mut self) -> Self {
        if self.argv.len() > 1 {
            let args: Vec<&str> = self.argv.split(" ").collect();
            self.command.args(&args[1..]);
        }
        self
    }
    fn add_stdout(mut self) -> Self {
        if let Some(ref string) = self.stdout {
            match File::open(string) {
                Ok(file) => {self.command.stdout(file);},
                Err(e) => println!("error{:?}", e),
            }
        }
        self
    }
    fn add_stderr(mut self) -> Self {
        if let Some(ref string) = self.stderr {
            match File::open(string) {
                Ok(file) => {self.command.stderr(file);},
                Err(e) => println!("error{:?}", e),
            }
        }
        self
    }
    fn add_env(mut self) -> Self {
        if let Some(ref vect) = self.env {
            let v: Vec<(String, String)> = vect.to_vec();
            self.command.envs(v);
        }
        self
    }
    fn spawn(mut self) -> Self {
        let now = Instant::now();
        let child = self.command.spawn();
        match child {
            Ok(mut child) => {
                println!("child {} launched with pid: {:?}", self.name, child.id());
                child.wait();
                let nownow = Instant::now();
                let duree = nownow.duration_since(now);
                println!("duree: {:?}", duree);
                if duree < self.starttime {
                    println!("must be restart");
                }
            }
            Err(e) => { println!("error {:?}", e);
            }
        }
        self
    }
    fn start(self) {
        self.add_args()
            .add_workingdir()
            .add_env()
            .add_stdout()
            .add_stderr()
            .spawn();
    }
}

fn exec_command (name: &Yaml, config: &Yaml) {
    let name = name.as_str().unwrap();
    let cmd = (&config["cmd"]).as_str().unwrap();
    let working_dir = (&config["workingdir"]).as_str();
    let autostart = (&config["autostart"]).as_bool();
    let env: Option<Vec<(String, String)>> = match (&config["env"]).as_hash() {
        Some(hash) => { Some(hash.iter()
                             .map(|(name, cmd)| {
                                 (String::from(name.as_str().unwrap()), 
                                  String::from(cmd.as_str().unwrap()))
                             }) //TODO: gerer les nombre
                             .collect())
        },
        None => None,
    };
    let stdout = (&config["stdout"]).as_str();
    let stderr = (&config["stderr"]).as_str();

    let exitcodes =  match (&config["exitcodes"]).as_vec() {
        Some(v) => Some(v.iter().map(|a| {
            a.as_i64().unwrap()})
                        .collect()),
        None => match (&config["exitcodes"]).as_i64() {
            Some(i) => Some(vec![i]),
            None => None,
        },
    };
    let startretries = (&config["startretries"]).as_i64();
    let umask = (&config["umask"]).as_i64();
    let autorestart = (&config["autorestart"]).as_i64();
    let starttime = (&config["starttime"]).as_i64();
    let stopsignal= (&config["stopsignal"]).as_i64();
    let stoptime = (&config["stoptime"]).as_i64();
    let numprocs = (&config["numprocs"]).as_i64();

    let mut process = Process::new(String::from(name),
                                   String::from(cmd),
                                   working_dir,
                                   autostart,
                                   env,
                                   stdout,
                                   stderr,
                                   exitcodes,
                                   startretries,
                                   umask,
                                   autorestart,
                                   starttime,
                                   stopsignal,
                                   stoptime,
                                   numprocs
                                  );
    println!("process is {:#?}", process);

    process.start();

}

fn parse_config_file (filename: &str)
{
    let mut f = File::open(filename).unwrap();

    let mut contents = String::new();

    f.read_to_string(&mut contents).unwrap();

    let docs = YamlLoader::load_from_str(&contents).unwrap();
    let doc = &docs[0];

    assert!(!doc["programs"].is_badvalue());

    let x = &doc["programs"];
    {
        let hash = x.as_hash().unwrap();
        //   println!("hash: {:#?}", hash);
        for (name, cmd) in hash.iter() {
            exec_command(name, cmd);
        }
    }
}

fn cli() {
    loop {
        print!("task_master> ");
        io::stdout().flush().unwrap();
        let mut guess = String::new();

        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        println!("You wrote: {}", guess);
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);
    let (option, filename) = parse_argv(&args);
    println!("{}, {}", option, filename);

    parse_config_file(filename);
    //    exec_command();
}

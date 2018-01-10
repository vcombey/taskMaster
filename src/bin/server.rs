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
#[allow(unused_imports)]
use std::process::Command;
#[allow(unused_imports)]
use std::process::Child;
use std::collections::HashMap;

extern crate yaml_rust;
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};

extern crate task_master;
use task_master::process::Config;
use task_master::process::Process;

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

fn launch_config(filename: &str) -> HashMap<String,Config> {
    let mut f = File::open(filename).unwrap();

    let mut contents = String::new();

    f.read_to_string(&mut contents).unwrap();

    let docs = YamlLoader::load_from_str(&contents).unwrap();
    let doc = &docs[0];

    assert!(!doc["programs"].is_badvalue());

    let mut map = HashMap::new();
    let program_section = &doc["programs"];
    {
        let hash = program_section.as_hash().unwrap();
        for (name, config) in hash.iter() {
            match (name.as_str(), config["cmd"].as_str()) {
                (Some(name), None) => eprintln!("Missing command for process {}", name),
                (None, Some(_)) => eprintln!("Missing process name"),
                (None, None) => eprintln!("Missing both process name and command"),
                (Some(name), Some(argv)) => {map.insert(String::from(name), Config::from_yaml(name, argv, config));},
            }
        }
    }
    return map;
}

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use task_master::cmd::Cmd;

fn lauch_processes(map: HashMap<String,Config>) -> HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)> {
    let mut threads: HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)> = HashMap::new();
    for (key, value) in map.iter() {
        let (sender, receiver) = channel();
        let clone_value = value.clone();
        let handle = thread::spawn(|| {
            let mut process = Process::new(clone_value, receiver);
            process.manage_program();
        });
        sender.send(Cmd::STOP);
        threads.insert(key.clone(), (handle, sender));
    }
    threads
}

extern crate liner;

use liner::Context;

const HELP_START : &'static str = "
start <name>		Start a process
start <gname>:*		Start all processes in a group
start <name> <name>	Start multiple processes or groups
start all		Start all processes
";

const HELP_RESTART : &'static str = "
restart <name>		Restart a process
restart <gname>:*	Restart all processes in a group
restart <name> <name>	Restart multiple processes or groups
restart all		Restart all processes
Note: restart does not reread config files. For that, see reread and update.
";

const HELP_STOP : &'static str = "
stop <name>		Stop a process
stop <gname>:*		Stop all processes in a group
stop <name> <name>	Stop multiple processes or groups
stop all		Stop all processes
";

const HELP_RELOAD : &'static str = "
reload 		Restart the remote supervisord.
";

const HELP_STATUS : &'static str = "
status <name>		Get status for a single process
status <gname>:*	Get status for all processes in a group
status <name> <name>	Get status for multiple named processes
status			Get all process status info
";

const HELP_SHUTDOWN : &'static str = "
shutdown 	Shut the remote supervisord down.
";

const HELP_DISPLAY : &'static str = "
default commands (type help <topic>):
=====================================
start  restart   stop  reload  status    shutdown
";

fn parse_cmd(line: &str) -> Option<(Cmd, String)> {
    let split: Vec<&str> = line.split(" ").collect();
    
    match split[0] {
        "help" => {
            let join = split[1..].join(" ");
            match &join[..] {
                "start" => println!("{}", HELP_START),
                "restart" => println!("{}", HELP_RESTART),
                "stop" => println!("{}", HELP_STOP),
                "reload" => println!("{}", HELP_RELOAD),
                "status" => println!("{}", HELP_STATUS),
                "shutdown" => println!("{}", HELP_SHUTDOWN),
                "" => println!("{}", HELP_DISPLAY),
                other => println!("*** No help on {}", other),
            }
            None
        },
        "stop" => Some((Cmd::STOP, String::from(split[1]))),
        /*"restart" => (Some(Cmd::RESTART),
        "start" => (Some(Cmd::START),
        "reload" => (Some(Cmd::RELOAD),
        "status" => (Some(Cmd::STATUS),
        "shutdown" => (Some(Cmd::SHUTDOWN),*/
        _ => {
            println!("*** Unknown syntax: {:?}", line);
            None
        },
    }
}

fn launch_cmd(threads: &mut HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)>, cmd: Cmd, arg: String) {
    if let Some(&(_, ref sender)) = threads.get(&arg) {
        sender.send(cmd);
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    let (option, filename) = parse_argv(&args);
    println!("{}, {}", option, filename);

    let map = launch_config(filename);
    //println!("map is {:#?}", map);
    let mut threads = lauch_processes(map);
    let mut con = Context::new();
    loop {
        use std::time::Duration;
        thread::sleep(Duration::from_secs(2));
        let res = con.read_line("task_master> ", &mut |_| {}).unwrap();

        if let Some((cmd, arg)) = parse_cmd(&res) {
            launch_cmd(&mut threads, cmd, arg);
        }
        if !res.is_empty() {
            con.history.push(res.into()).unwrap();
        }
    }
    /*for (key, &(ref handle, ref sender)) in threads.iter() {
      handle.join();
      }*/
}

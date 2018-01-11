#[allow(unused_imports)]
use std::io;
#[allow(unused_imports)]
use std::io::Write;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)] // REMOVE
use std::fs::File; // REMOVE
#[allow(unused_imports)] // REMOVE
use std::io::prelude::*; // REMOVE
#[allow(unused_imports)]
use std::process::Command;
#[allow(unused_imports)]
use std::process::Child;
use std::collections::HashMap;


extern crate task_master;
use task_master::tm_mod::service::thread::process::Process;
use task_master::tm_mod::config::Config;

fn parse_argv (args: &[String]) -> (&str, &str)
{
    if args.len() < 3 {
        panic!("Not enough arguments");
    }
    let option = &args[1];
    if option != "-c" {
        panic!(format!("Unknown option: -- {}", option));
    }
    let filename = &args[2];

    (option, filename)
}

fn hash_config(task_master: &TmStruct) -> HashMap<String, HashMap<String,Config>> {
    /// Reads the config file using TmStruct methods, and turns it
    /// into a HashMap representing the structure of the services and
    /// programm we need to launch. Multiple service cannot have the
    /// same name, and multiple process cannot have the same name EVEN
    /// ACROSS different services, and finally a process cannot have
    /// the same name a service does. 0 ambiguity allowed.
    let doc = task_master.parse_config_file().unwrap();
    let doc = &doc[0];
    let doc = doc.as_hash().unwrap();

    let mut taken_process_names: Vec<String> = Vec::new();


    // Big map build
    let mut big_map = HashMap::new();
    for (section_name_yaml, section_yaml) in doc.iter() {
        let section_name = section_name_yaml.as_str().unwrap();
        let section_hash = section_yaml.as_hash().unwrap();

        // Litle map build
        let mut little_map = HashMap::new();
        for (name, config) in section_hash.iter() {
            match (name.as_str(), config["cmd"].as_str()) {
                (Some(name), None) => eprintln!("Missing command for process {}", name),
                (None, Some(_)) => eprintln!("Missing process name"),
                (None, None) => eprintln!("Missing both process name and command"),
                (Some(name), Some(argv)) => {

                    //  Check if a service/process with the same name aready exists
                    if big_map.contains_key(name) {
                        eprintln!("Cannot create process of the name '{}': a service of the same name already exists", name);
                        panic!("Need to improve this server.c");
                    } else if taken_process_names.contains(&String::from(name)) {
                        eprintln!("Cannot create process of the name '{}': a process of the same name already exists", name);
                        panic!("Need to improve this server.c");
                    }

                    // Insert into little map
                    little_map.insert(String::from(name), Config::from_yaml(name, argv, config));
                    taken_process_names.push(String::from(name));
                },
            }

        }
        // Check if a service / process with the same name already exists
        if big_map.contains_key(section_name) {
            eprintln!("Cannot create service of the name '{}': a service of the same name already exists", section_name);
            panic!("Need to improve this server.c");
        } else if taken_process_names.contains(&String::from(section_name)) {
            eprintln!("Cannot create service of the name '{}': a process of the same name already exists", section_name);
            panic!("Need to improve this server.c");
        }

        // Insert into big map
        big_map.insert(String::from(section_name), little_map);
    }
    return big_map;
}

use std::sync::mpsc::channel;
use std::sync::mpsc::Sender;
use std::thread;
use task_master::tm_mod::cmd::Cmd;

fn lauch_processes(map: HashMap<String,Config>) -> HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)> {
    let mut threads: HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)> = HashMap::new();
    for (key, value) in map.iter() {
        let (sender, receiver) = channel();
        let clone_value = value.clone();
        let handle = thread::spawn(|| {
            let mut process = Process::new(clone_value, receiver);
            process.manage_program();});
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

use task_master::tm_mod::TmStruct;

fn main()
{
    let args: Vec<String> = env::args().collect();
    let (option, filename) = parse_argv(&args);
    let mut tm = TmStruct::new(filename);

    let map = hash_config(&tm);
    //println!("map is {:#?}", map);
    // let mut threads = lauch_processes(map);
    tm.launch_from_hash(map);
    // let mut con = Context::new();
    loop {
        // use std::time::Duration;
        // thread::sleep(Duration::from_secs(2));
        // let res = con.read_line("task_master> ", &mut |_| {}).unwrap();

        // if let Some((cmd, arg)) = parse_cmd(&res) {
        //     launch_cmd(&mut threads, cmd, arg);
        // }
        // if !res.is_empty() {
        //     con.history.push(res.into()).unwrap();
        // }
    }
    /*for (key, &(ref handle, ref sender)) in threads.iter() {
    handle.join();
}*/
}

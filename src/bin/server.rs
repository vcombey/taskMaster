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
use task_master::process::execute_process;

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
use std::thread;
use task_master::cmd;

fn lauch_processes(map: HashMap<String,Config>) {
    let mut threads: Vec<thread::JoinHandle<()>> = Vec::new();
    for (key, value) in map.iter() {
        let (sender, receiver) = channel();
        let clone_value = value.clone();
        let handle = thread::spawn(|| {
            execute_process(Process::new(clone_value, receiver));
        });
        sender.send(cmd::Cmd::STOP);
        threads.push(handle);
    }
    for handle in threads {
        handle.join();
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();
    let (option, filename) = parse_argv(&args);
    println!("{}, {}", option, filename);

    let map = launch_config(filename);
    //println!("map is {:#?}", map);
    lauch_processes(map);
}

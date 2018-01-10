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
use std::sync::mpsc::Receiver;
use std::thread;
use task_master::cmd::Cmd;

fn lauch_processes(map: HashMap<String,Config>) -> (HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)>, Receiver<String>){
    let mut threads: HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)> = HashMap::new();
    let (sender_to_main, receiver_from_threads) = channel();
    for (key, value) in map.iter() {
        let (sender, receiver) = channel();
        let clone_value = value.clone();
        let sender_to_main_clone = sender_to_main.clone();
        let handle = thread::spawn(move || {
            let mut process = Process::new(clone_value, receiver, sender_to_main_clone);
            process.manage_program();
        });
        threads.insert(key.clone(), (handle, sender));
    }
    (threads, receiver_from_threads)
}

extern crate liner;

use liner::Context;

fn launch_cmd(threads: &mut HashMap<String,(thread::JoinHandle<()>, Sender<Cmd>)>, cmd: Cmd, arg: &str) {
    if let Some(&(_, ref sender)) = threads.get(arg) {
        sender.send(cmd);
    }
}

use std::time::Duration;
use std::net::{TcpListener, TcpStream};
use task_master::tcp;
use task_master::cli::parse_cmd;

fn main()
{
    let listening_stream = TcpListener::bind("127.0.0.1:4242")
        .unwrap();

    //let emit_stream = TcpStream::connect("127.0.0.1:4242")
     //   .unwrap();

    tcp::receive(listening_stream);

    let args: Vec<String> = env::args().collect();
    let (option, filename) = parse_argv(&args);
    println!("{}, {}", option, filename);

    let map = launch_config(filename);
    //println!("map is {:#?}", map);
    let (mut threads, receiver) = lauch_processes(map);
    let mut con = Context::new();
    loop {
        thread::sleep(Duration::from_secs(2));

        match receiver.try_recv() {
            Ok(mess) => {
                eprintln!("mess receive {}", mess);
            },
            Err(e) => { eprintln!("{:?}", e); },
        }
        let res = con.read_line("task_master> ", &mut |_| {}).unwrap();

        if let Some((cmd, arg)) = parse_cmd(&res) {
            launch_cmd(&mut threads, cmd, arg[0]);
        }
        if !res.is_empty() {
            con.history.push(res.into()).unwrap();
        }
    }
    /*for (key, &(ref handle, ref sender)) in threads.iter() {
      handle.join();
      }*/
}

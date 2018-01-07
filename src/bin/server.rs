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

extern crate yaml_rust;
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};

extern crate task_master;
use task_master::task_master::Process;
    
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

fn exec_command (name: &Yaml, config: &Yaml) {
    let name = name.as_str().unwrap();
    let process = Process::from_yaml(name, config);

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

fn main()
{
    let args: Vec<String> = env::args().collect();
    let (option, filename) = parse_argv(&args);
    println!("{}, {}", option, filename);

    parse_config_file(filename);
}

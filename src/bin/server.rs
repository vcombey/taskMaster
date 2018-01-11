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
use task_master::tm_mod::TmStruct;

fn main()
{
    let args: Vec<String> = env::args().collect();
    let (option, filename) = parse_argv(&args);
    let mut tm = TmStruct::new(filename);

    let map = tm.hash_config();
    //println!("map is {:#?}", map);
    tm.launch_from_hash(map);
    loop {
    }
}

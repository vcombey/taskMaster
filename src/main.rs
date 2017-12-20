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

use std::process::Command;

fn exec_command() {
    Command::new("./wait_and_print").spawn().unwrap();
    Command::new("ls").arg("-l").spawn().unwrap();
    let mut child = Command::new("cat").spawn().unwrap();
    
    child.wait().unwrap();
}

fn parse_argv(args: &[String]) -> (&str, &str)
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

fn parse_config_file(filename: &str)
{
    let mut f = File::open(filename).expect("file not found");

    let mut contents = String::new();

    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");
    
    println!("{}", contents);

    let docs = YamlLoader::load_from_str(&contents).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];


    println!("{:?}", doc);
    // Debug support
    /*
    println!("{:?}", doc);
    println!("{:?}", doc["programs"]["nginx"]);
    for (key, value) in doc.iter() {
        println!("{:?}, {:?}",key, value);
    }
    */

//   println!("{:?}", doc[1]);

    match doc {
        &Yaml::Hash(ref a) => println!("{:?}", a),
    /*
        Real(a) => println!("{:?}", a),
        Integer(&a) => println!("{:?}", a),
        String(&a) => println!("{:?}", a),
        Boolean(&a) => println!("{:?}", a),
        Array(&a) => println!("{:?}", a),
       */ 
        /*
        Alias(&a) => println!("{:?}", a),
        Null => println!("null"),
        BadValue => println!("badValue"),
        */
    //    Yaml::Real(&a) => println!("{:?}", a),
//        Yaml::Array(&a) => println!("{:?}", a),
        _ => println!("autre"),
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
    let (query, filename) = parse_argv(&args);
    println!("{}, {}", query, filename);

    parse_config_file(filename);
    exec_command();
}

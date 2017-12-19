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
use yaml_rust::{YamlLoader, YamlEmitter};

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

fn main()
{
    let args: Vec<String> = env::args().collect();
   // println!("{:?}", args);
    let (query, filename) = parse_argv(&args);
    println!("{}, {}", query, filename);

    let mut f = File::open(filename).expect("file not found");

    let mut contents = String::new();

    f.read_to_string(&mut contents)
        .expect("something went wrong reading the file");
    
    println!("{}", contents);

    let docs = YamlLoader::load_from_str(&contents).unwrap();

    // Multi document support, doc is a yaml::Yaml
    let doc = &docs[0];

    // Debug support
    println!("{:?}", doc);

    // Index access for map & array
    //assert_eq!(doc["bar"][1].as_f64().unwrap(), 2.0);

    // Chained key/array access is checked and won't panic,
    // return BadValue if they are not exist.
    //assert!(doc["INVALID_KEY"][100].is_badvalue());

    // Dump the YAML object
    /*let mut out_str = String::new();
    {
        let mut emitter = YamlEmitter::new(&mut out_str);
        emitter.dump(doc).unwrap(); // dump the YAML object to a String
    }
    println!("{}", out_str);
    loop {
        print!("task_master> ");
        io::stdout().flush().unwrap();
        let mut guess = String::new();

        io::stdin().read_line(&mut guess)
            .expect("Failed to read line");

        println!("You wrote: {}", guess);
    }
    */
}

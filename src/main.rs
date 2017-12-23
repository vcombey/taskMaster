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
//extern crate linked_hash_map;
//use linked_hash_map::LinkedHashMap;



fn exec_command() {
    
    println!("{:?}", env::current_dir().unwrap());
   /* Command::new("./wait_and_print").spawn().unwrap();
    Command::new("ls").arg("-l").spawn().unwrap();
    let mut child = Command::new("cat").spawn().unwrap();

    child.wait().unwrap();
    */
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


    println!("{:#?}", doc);
    //   println!("{:?}", doc[1]);

    match doc {
        &Yaml::Array(ref a) => println!("a is {:?}", a),
        &Yaml::Hash(ref a) => {
            println!("{:?}", a);
            /*
            for (name, yaml) in a.iter().collect() {
                println!("name is : {:?}\n yaml is : {:?} ", name, yaml);
            }
            */
        },
        _ => println!("autre"),
    }
    if let Some(h) = doc.as_hash() {
        println!("h is {:?}", h);
    }
    /*
    let mut map = Yaml::Hash("lol");
    map.insert(Yaml::from_str("lala"), Yaml::from_str("lolo"));
    //map.insert(2, "b");
    let h = Yaml::Hash(map);
    for (name, yaml) in h.iter().collect() {
        println!("name is : {:?} yaml is : {:?} ", name, yaml);
    }
    */
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
        exec_command();
}

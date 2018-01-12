use std::env;

extern crate task_master;

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

    // Unused variable right here
    let (_option, filename) = parse_argv(&args); // UNUSED
    let mut tm = TmStruct::new(filename);

    let map = tm.hash_config();
    //println!("map is {:#?}", map);
    tm.launch_from_hash(map);
    loop {
    }
}

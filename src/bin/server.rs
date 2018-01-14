extern crate task_master;
use std::env;
use task_master::tm_mod::TmStruct;
use task_master::tm_mod::cmd::Cmd;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

fn parse_argv (args: &[String]) -> (&str, &str) {
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
use std::net::{TcpStream,TcpListener};
use std::io::{Read, Write};

pub fn receive(stream: &mut TcpStream) -> Cmd {
    let mut buffer = [0; 512];

    let nb_bytes = stream.read(&mut buffer).unwrap();
    let request = &buffer[..nb_bytes];
 //   println!("Request: {:?} {:?}", nb_bytes, String::from_utf8_lossy(request));
    return serde_json::from_str(&String::from_utf8_lossy(request)).unwrap();
}

fn handle_connection(mut stream: TcpStream, tm: &mut TmStruct) {
    let cmd = receive(&mut stream);

    //let mut buffer = [0; 512];
    //stream.read(&mut buffer).unwrap();

    //println!("Request: {:?}", cmd);

    if let Err(e) = tm.exec_cmd(cmd) {
        let response = format!("{}", e);

        stream.write(response.as_bytes()).unwrap();
    }
}

fn server(port: &str, tm: &mut TmStruct)
{
    let listener = TcpListener::bind(port).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream, tm);
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();

    // Unused variable right here
    let (_option, filename) = parse_argv(&args); // UNUSED
    let mut tm = TmStruct::new(filename);

    let map = tm.hash_config();
    tm.launch_from_hash(map);
    server("127.0.0.1:8080", &mut tm);
    loop {
    }
}

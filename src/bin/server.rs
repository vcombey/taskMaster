extern crate task_master;
use std::env;
use task_master::tm_mod::TmStruct;
use task_master::tm_mod::cmd::Cmd;
use task_master::tm_mod::cmd::Instruction;

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

fn handle_connection(mut stream: TcpStream, tm: &mut TmStruct) -> Result<(), ()> {
    let cmd = receive(&mut stream);

    //let mut buffer = [0; 512];
    //stream.read(&mut buffer).unwrap();

    if cmd.instruction == Instruction::SHUTDOWN {
        return Err(());
    }
    //println!("Request: {:?}", cmd);
    
    let response = tm.try_receive_from_threads()
        .unwrap_or(String::from("pb receiving from threads"));
    let response_err = match tm.exec_cmd(cmd) {
        Err(e) => format!("{}", e),
        Ok(_) => format!(""),
    };
    let response = format!("{}{}", response, response_err);
    stream.write(response.as_bytes()).unwrap();
    Ok(())
}

fn server(port: &str, tm: &mut TmStruct) -> Result<(), ()> {
    let listener = TcpListener::bind(port).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        if handle_connection(stream, tm).is_err() {
            return Err(());
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Unused variable right here
    let (_option, filename) = parse_argv(&args); // UNUSED
    let mut tm = TmStruct::new(filename);

    let map = tm.hash_config();
    tm.launch_from_hash(map);
    server("127.0.0.1:8080", &mut tm);
}

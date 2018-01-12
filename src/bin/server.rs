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
use std::net::{TcpStream,TcpListener};
use std::io::{Read, Write};

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];

    stream.read(&mut buffer).unwrap();
    let response = "HTTP/1.1 200 OK\r\n\r\n";

    stream.write(response.as_bytes()).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));
}

fn server(port: &str)
{
    let listener = TcpListener::bind(port).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        handle_connection(stream);
    }
}

fn main()
{
    let args: Vec<String> = env::args().collect();

    server("127.0.0.1:8080");
    // Unused variable right here
    let (_option, filename) = parse_argv(&args); // UNUSED
    let mut tm = TmStruct::new(filename);

    let map = tm.hash_config();
    //println!("map is {:#?}", map);
    tm.launch_from_hash(map);
    loop {
    }
}

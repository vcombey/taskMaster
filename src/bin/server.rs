extern crate task_master;

extern crate serde;
extern crate serde_json;

use std::env;
use std::net::{TcpStream,TcpListener};
use std::io::{Read, Write};
use std::time::Duration;

use task_master::tm_mod::TmStruct;
use task_master::tm_mod::cmd::Cmd;
use task_master::tm_mod::cmd::Instruction;


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

pub fn receive(stream: &mut TcpStream) -> Cmd {
    let mut buffer = [0; 512];

    let nb_bytes = stream.read(&mut buffer).unwrap();
    let request = &buffer[..nb_bytes];
 //   println!("Request: {:?} {:?}", nb_bytes, String::from_utf8_lossy(request));
    return serde_json::from_str(&String::from_utf8_lossy(request)).unwrap();
}

fn handle_connection(mut stream: TcpStream, tm: &mut TmStruct) -> Result<(), ()> {
    let cmd = receive(&mut stream);

    if cmd.instruction == Instruction::SHUTDOWN {
        return Err(());
    }
    if cmd.instruction == Instruction::REREAD {
        tm.reread();
    }

    let mut nb_receive = 0;
    let response_err = match tm.exec_cmd(cmd, &mut nb_receive) {
        Err(e) => format!("{}", e),
        Ok(_) => format!(""),
    };
    let response = tm.try_receive_from_threads(nb_receive, Duration::from_secs(2))
        .unwrap_or(String::from("pb receiving from threads"));
    let response = format!("{}{}", response, response_err);
    stream.write(response.as_bytes()).unwrap();
    Ok(())
}

fn launch_server(port: &str, tm: &mut TmStruct) -> Result<(), ()> {
    let listener = TcpListener::bind(port).unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        if handle_connection(stream, tm).is_err() {
            return Err(());
        }
    }
    Ok(())
}
// extern crate daemonize;

// use daemonize::{Daemonize};

fn main() {
    /*let daemonize = Daemonize::new()
        .pid_file("/tmp/test.pid") // Every method except `new` and `start`
        .chown_pid_file(true)      // is optional, see `Daemonize` documentation
        .working_directory("/tmp") // for default behaviour.
        .user("nobody")
        .group("daemon") // Group name
        .group(2)        // Or group id
        .privileged_action(|| "Executed before drop privileges");

    match daemonize.start() {
        Ok(_) => println!("Success, daemonized"),
        Err(e) => eprintln!("{}", e),
    }
    */

    let args: Vec<String> = env::args().collect();

    // Unused variable right here
    let (_option, filename) = parse_argv(&args); // UNUSED
    let mut tm = TmStruct::new(filename);

    let map = tm.hash_config();
    tm.launch_from_hash(map);
    let _ = launch_server("127.0.0.1:8080", &mut tm);
}

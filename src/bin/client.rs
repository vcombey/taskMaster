extern crate task_master;
extern crate liner;

use liner::Context;
use task_master::tm_mod::cmd::Cmd;
use task_master::cli;


fn parse_into_cmd(line: &str) -> Option<Cmd> {
    let mut split: Vec<&str> = line.split_whitespace().collect();

    match split.get(0) {
        Some(&"help") => {
            match split.get(1) {
                Some(value) => match *value {
                    "start" => {println!("{}", cli::HELP_START);None},
                    "restart" => {println!("{}", cli::HELP_RESTART);None},
                    "stop" => {println!("{}", cli::HELP_STOP);None},
                    "reload" => {println!("{}", cli::HELP_RELOAD);None},
                    "status" => {println!("{}", cli::HELP_STATUS);None},
                    "shutdown" => {println!("{}", cli::HELP_SHUTDOWN);None},
                    "" => {println!("{}", cli::HELP_DISPLAY);None},
                    _ => {println!("{}", cli::HELP_DISPLAY);None},
                },
                None => {println!("{}", cli::HELP_DISPLAY);None},
            }
        },
        Some(_) => { // Parse and discard error
            Cmd::from_vec(split).map_err(|e| eprintln!("{}", e))
                .ok()
                },
        None => None,
    }
}

use std::net::{TcpStream,TcpListener};
use std::io::{Write, Read};

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

use serde::{Serialize, Serializer};

fn emit<T>(stream: &mut TcpStream, t: T)
    where T: serde::Serialize + serde::export::fmt::Debug
{
    println!("Before serialize, on the emit side : {:?}", t);
    let serialized : String = serde_json::to_string(&t).unwrap();
    stream.write(&serialized.as_bytes());
    //stream.flush().unwrap();
}


fn main() {
    let mut con = Context::new();
    loop {
        let res = con.read_line("task_master> ", &mut |_| {}).unwrap();
        let cmd = parse_into_cmd(&res);
        if cmd.is_some() {
            let mut buffer = String::new();
            let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
            emit(&mut stream, cmd);
            let _ = stream.read_to_string(&mut buffer);
            println!("{}", buffer);
        }
        con.history.push(res.into());
    }
}

#[cfg(test)]
pub mod test_parse_into_cmd{
    use super::*;
    use task_master::tm_mod::cmd::*;

    #[test]
    fn cmd_empty() {
        assert_eq!(parse_into_cmd(""), None);
        assert_eq!(parse_into_cmd("          "), None);
    }

    #[test]
    fn help() {
        assert_eq!(parse_into_cmd("help"), None);
        assert_eq!(parse_into_cmd("help fsfsd"), None);
        assert_eq!(parse_into_cmd("help status"), None);
    }

    #[test]
    fn double_point() {
        assert_eq!(parse_into_cmd(":::::::"), None);
        assert_eq!(parse_into_cmd("lol::lol"), None);
        assert_eq!(parse_into_cmd(":"), None);
        assert_eq!(parse_into_cmd("lala:"), None);
        assert_eq!(parse_into_cmd(":lala"), None);
    }

    #[test]
    fn test_cmd_parse_many_process() {
        assert_eq!(parse_into_cmd("     start     process_one    process_two").unwrap(),
        Cmd::new(Instruction::START,
                 vec![Target::Process("process_one".to_string()), Target::Process("process_two".to_string())],
                 ));
    }

    #[test]
    fn test_cmd_mix() {
        assert_eq!(parse_into_cmd("start process_one service_one:process_two").unwrap(),
        Cmd::new(Instruction::START,
                 vec![Target::Process("process_one".to_string()), Target::ServiceProcess(("service_one".to_string(), "process_two".to_string()))],
                 ));
    }
}


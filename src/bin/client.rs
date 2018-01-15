extern crate task_master;
extern crate liner;

extern crate serde;
extern crate serde_json;

use liner::Context;

use std::net::TcpStream;
use std::io::{Write, Read};

use task_master::tm_mod::cmd::Cmd;
use task_master::cli;
use std::error::Error;

fn parse_into_cmd(line: &str) -> Option<Cmd> {
    let split: Vec<&str> = line.split_whitespace().collect();

    match split.get(0) {
        Some(&"help") => {
            match split.get(1) {
                Some(value) => match *value {
                    "start" => println!("{}", cli::HELP_START),
                    "restart" => println!("{}", cli::HELP_RESTART),
                    "stop" => println!("{}", cli::HELP_STOP),
                    "reload" => println!("{}", cli::HELP_RELOAD),
                    "status" => println!("{}", cli::HELP_STATUS),
                    "shutdown" => println!("{}", cli::HELP_SHUTDOWN),
                    "" => println!("{}", cli::HELP_DISPLAY),
                    _ => println!("{}", cli::HELP_DISPLAY),
                },
                None => println!("{}", cli::HELP_DISPLAY),
            };
            None
        },
        Some(_) => { // Parse and discard error
            Cmd::from_vec(split).map_err(|e| eprintln!("{}", e))
                .ok()
                },
        None => None,
    }
}


fn emit<T>(stream: &mut TcpStream, t: T)
    where T: serde::Serialize + serde::export::fmt::Debug
{
    println!("cmd : {:?}", t);
    let serialized : String = serde_json::to_string(&t).unwrap();
    stream.write(&serialized.as_bytes()).unwrap();
    //stream.flush().unwrap();
}


fn main() {
    let mut con = Context::new();
    loop {
        let res = match con.read_line("task_master> ", &mut |_| {}) {
            Ok(res) => res,
            Err(_) => { return; },
        };
        let cmd = parse_into_cmd(&res);
        if cmd.is_some() {
            let mut buffer = String::new();
            //println!("cmd : {:#?}", cmd);
            match TcpStream::connect("127.0.0.1:8080") {
                Ok(mut stream) => {
                    emit(&mut stream, cmd);
                    let _ = stream.read_to_string(&mut buffer);
                    println!("{}", buffer);
                },
                Err(e) => eprintln!("{}", e.description()),
            }
        }
        con.history.push(res.into()).unwrap();
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


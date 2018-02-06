extern crate liner;
extern crate task_master;

extern crate serde;
extern crate serde_json;

use liner::Context;

use std::net::TcpStream;
use std::io::{Read, Write};

use task_master::tm_mod::cmd::Cmd;
use task_master::cli;
use task_master::tm_mod::error_utils::print_err;
use std::error::Error;

fn parse_into_cmd(line: &str) -> Option<Cmd> {
    let split: Vec<&str> = line.split_whitespace().collect();

    match split.get(0) {
        Some(&"help") => {
            match split.get(1) {
                Some(value) => match *value {
                    "start" => eprintln!("{}", cli::HELP_START),
                    "restart" => eprintln!("{}", cli::HELP_RESTART),
                    "stop" => eprintln!("{}", cli::HELP_STOP),
                    "reload" => eprintln!("{}", cli::HELP_RELOAD),
                    "status" => eprintln!("{}", cli::HELP_STATUS),
                    "shutdown" => eprintln!("{}", cli::HELP_SHUTDOWN),
                    "" | _ => eprintln!("{}", cli::HELP_DISPLAY),
                },
                None => eprintln!("{}", cli::HELP_DISPLAY),
            };
            None
        }
        Some(_) => {
            // Parse and discard error
            Cmd::from_vec(&split).map_err(|e| eprintln!("{}", e)).ok()
        }
        None => None,
    }
}

fn emit<T>(stream: &mut TcpStream, t: T) -> Result<(), String>
where
    T: serde::Serialize + serde::export::fmt::Debug,
{
    //eprintln!("cmd : {:?}", t);
    let serialized: String = serde_json::to_string(&t).map_err(|e| format!("{}", e))?;
    stream
        .write(serialized.as_bytes())
        .map_err(|e| format!("{}", e))?;
    Ok(())
}

fn main() {
    let mut con = Context::new();
    loop {
        let res = match con.read_line("task_master> ", &mut |_| {}) {
            Ok(line) => line,
            Err(_) => return,
        };
        if let Some(cmd) = parse_into_cmd(&res) {
            let mut buffer = String::new();
            //eprintln!("cmd : {:#?}", cmd);
            match TcpStream::connect("127.0.0.1:4242") {
                Ok(mut stream) => match emit(&mut stream, cmd) {
                    Ok(_) => {
                        if let Err(e) = stream.read_to_string(&mut buffer) {
                            eprintln!("{}", e.description());
                        }
                        eprintln!("{}", buffer);
                    }
                    Err(e) => eprintln!("{}", e),
                },
                Err(e) => eprintln!("{}", e.description()),
            }
        }
        if !res.as_str().split_whitespace().next().is_none() {
            print_err(con.history.push(res.into()));
        }
    }
}

#[cfg(test)]
pub mod test_parse_into_cmd {
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
        assert_eq!(
            parse_into_cmd("     start     process_one    process_two").unwrap(),
            Cmd::new(
                Instruction::START,
                vec![
                    Target::Process("process_one".to_string(), None),
                    Target::Process("process_two".to_string(), None),
                ],
            )
        );
    }

    #[test]
    fn test_cmd_mix() {
        assert_eq!(
            parse_into_cmd("start process_one service_one:process_two").unwrap(),
            Cmd::new(
                Instruction::START,
                vec![
                    Target::Process("process_one".to_string(), None),
                    Target::ServiceProcess((
                        "service_one".to_string(),
                        "process_two".to_string(),
                        None,
                    )),
                ],
            )
        );
    }

    #[test]
    fn test_cmd_no_target() {
        assert_eq!(
            parse_into_cmd("start").unwrap(),
            Cmd::new(Instruction::START, vec![Target::ALL],)
        );
    }
}

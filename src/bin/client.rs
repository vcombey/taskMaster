extern crate task_master;
extern crate liner;

use liner::Context;
use task_master::tm_mod::cmd::Cmd;
use task_master::tm_mod::cmd::Instruction;
use task_master::cli::*;


fn parse_into_cmd(line: &str) -> Option<Cmd> {
    let mut split = line.split(" ");

    let first = split.next()?;
    match first
    {
        "help" => {
            match split.next() {
                Some(value) => match value {
                    "start" => println!("{}", HELP_START),
                    "restart" => println!("{}", HELP_RESTART),
                    "stop" => println!("{}", HELP_STOP),
                    "reload" => println!("{}", HELP_RELOAD),
                    "status" => println!("{}", HELP_STATUS),
                    "shutdown" => println!("{}", HELP_SHUTDOWN),
                    &_ => println!("no help for that"),
                },
                None => println!("{}", HELP_DISPLAY),
            };
            None
        },
        _ => {
            match Cmd::from_line(line) {
                Ok(cmd) => Some(cmd),
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
            }
        },
    }

    /*
    */
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
        //println!("{:?}", cmd);
        con.history.push(res.into());
        let mut buffer = String::new();
        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        emit(&mut stream, cmd);
        let _ = stream.read_to_string(&mut buffer);
        println!("message get{}", buffer);
    }
}

#[cfg(test)]
mod test {
    use parse_into_cmd;
    use task_master::tm_mod::cmd::Cmd;
    use task_master::tm_mod::cmd::Instruction;
    use task_master::tm_mod::cmd::Target;

#[test]
    fn cmd_vide() {
        assert_eq!(parse_into_cmd(""), None);
    }
#[test]
    fn help() {
        assert_eq!(parse_into_cmd("help"), None);
        assert_eq!(parse_into_cmd("help fsfsd"), None);
        assert_eq!(parse_into_cmd("help status"), None);
    }
#[test]
    fn one_cmd() {
        let instruction_vect :Vec<(&str, Instruction)> = 
            vec![("start" , Instruction::START),
            ("restart" , Instruction::RESTART),
            ("stop" , Instruction::STOP),
            ("reload" , Instruction::RELOAD),
            ("status" , Instruction::STATUS),
            ("shutdown" , Instruction::SHUTDOWN)];

        for (ins_str, ins) in instruction_vect {
            assert_eq!(parse_into_cmd(&format!("{} {}", ins_str, "cmd1")), 
                       Some(Cmd::new(ins, 
                                     vec![Target::Process(String::from("cmd1"))])));

            assert_eq!(parse_into_cmd(&format!("{} {}:{}", ins_str, "serv1", "cmd1")),
            Some(Cmd::new(ins,
                          vec![Target::ServiceProcess(
                              (String::from("serv1"), String::from("cmd1")))])));

            assert_eq!(parse_into_cmd(&format!("{} {}:*", ins_str, "serv1")),
            Some(Cmd::new(ins,
                          vec![Target::Service(
                              String::from("serv1"))])));

            assert_eq!(parse_into_cmd(ins_str), 
                       Some(Cmd::new(ins,
                                     Vec::new())));
        }
    }
#[test]
    fn double_point() {
        assert_eq!(parse_into_cmd(":::::::"), None);
        assert_eq!(parse_into_cmd(":"), None);
        assert_eq!(parse_into_cmd("lala:"), None);
        assert_eq!(parse_into_cmd(":lala"), None);
    }
}


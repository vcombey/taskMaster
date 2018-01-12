extern crate task_master;
extern crate liner;

use liner::Context;
use task_master::tm_mod::cmd::Cmd;
use task_master::cli;


fn parse_into_cmd(line: &str) -> Option<Cmd> {
    let split = line.split(" ");

    match split.next() {
        Some("help") => {
            match split.next () {
                Some(value) => match value {
                    "start" => println!("{}", HELP_START),
                    "restart" => println!("{}", HELP_RESTART),
                    "stop" => println!("{}", HELP_STOP),
                    "reload" => println!("{}", HELP_RELOAD),
                    "status" => println!("{}", HELP_STATUS),
                    "shutdown" => println!("{}", HELP_SHUTDOWN),
                    "" => println!("{}", HELP_DISPLAY),
                },
                None => println!("{}", HELP_DISPLAY),
            },
            Some(_) => {
                match Cmd::from_iterator(cmd, split) {
                    Ok(cmd) => Some(cmd),
                    Err(e) => {
                        eprintln!("{}", e);
                        None
                    }
                }
            },
            None => None
        }

    }
}

fn main() {
    let mut con = Context::new();
    loop {
        let _res = con.read_line("task_master> ", &mut |_| {}).unwrap();
    }
}

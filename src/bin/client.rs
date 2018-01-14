extern crate task_master;
extern crate liner;

use liner::Context;
use task_master::tm_mod::cmd::Cmd;
use task_master::cli;


fn parse_into_cmd(line: &str) -> Option<Cmd> {
    let line: Vec<&str> = line.split(" ").collect();

    match line.get(0) {
        Some(&"help") => {
            match line.get(1) {
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
        Some(_) => {
            match Cmd::from_vec(line) {
                Some(ret) => match ret {
                    Ok(cmd) => Some(cmd),
                    Err(e) => {
                        eprintln!("{}", e);
                        None
                    }
                },
                None => None,
            }
        },
        None => None,
    }
}

fn main() {
    let mut con = Context::new();
    loop {
        let _res = con.read_line("task_master> ", &mut |_| {}).unwrap();
    }
}

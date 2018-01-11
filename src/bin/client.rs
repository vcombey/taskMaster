extern crate task_master;
use task_master::tcp;
use task_master::cli::parse_cmd;
use std::net::{TcpListener, TcpStream};
use task_master::tcp::TcpTwoSide;

extern crate liner;
use liner::Context;


fn main() {
    let mut con = Context::new();
    let mut tcp_two_side = TcpTwoSide::new("127.0.0.1:8082", "127.0.0.1:8080");
    loop {
        let res = con.read_line("task_master> ", &mut |_| {}).unwrap();

        if let Some(cmdargs) = parse_cmd(&res) {
            tcp_two_side.emit(cmdargs);
        }
        if !res.is_empty() {
            con.history.push(res.into()).unwrap();
        }
    }
}

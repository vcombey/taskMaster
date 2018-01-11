extern crate task_master;
#[allow(unused_imports)]
use std::net::{TcpListener, TcpStream};

extern crate liner;
use liner::Context;


fn main() {
    let mut con = Context::new();
    loop {
        let _res = con.read_line("task_master> ", &mut |_| {}).unwrap();
    }
}

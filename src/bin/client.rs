extern crate task_master;
use task_master::tcp;
use task_master::cli::parse_cmd;
use std::net::{TcpListener, TcpStream};

extern crate liner;
use liner::Context;


fn main() {
    let mut con = Context::new();
    let mut emit_stream = TcpStream::connect("127.0.0.1:4242");
    if emit_stream.is_err() {
        eprintln!("connection refused");
    }
    loop {
        let res = con.read_line("task_master> ", &mut |_| {}).unwrap();

        if let Some(cmdargs) = parse_cmd(&res) {
            if let Ok(ref mut emit_stream) = emit_stream {
                tcp::emit(emit_stream, cmdargs);
            } else {
                eprintln!("connection refused");
            }

        }
        if !res.is_empty() {
            con.history.push(res.into()).unwrap();
        }
    }
}

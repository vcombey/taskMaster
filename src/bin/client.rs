extern crate task_master;
use task_master::tcp;
use std::net::{TcpListener, TcpStream};

fn main() {
    println!("this is the client");
    let emit_stream = TcpStream::connect("127.0.0.1:4242")
        .unwrap();
    tcp::emit(emit_stream);
}

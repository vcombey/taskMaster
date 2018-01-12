use std::net::{TcpStream,TcpListener};
use std::io::{Write, Read};

fn main() {
  let mut buffer = String::new();
  let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
  let _ = stream.write(b"lalala");
    let _ = stream.read_to_string(&mut buffer);
    println!("message get{}", buffer);
}

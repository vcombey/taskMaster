use serde;

use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read};
use serde::{Serialize, Serializer};
use serde_json;
use cmd::Cmd;
use cli::CmdArgs;

pub fn emit<T>(stream: &mut TcpStream, t: T)
    where T: serde::Serialize,
          T: serde::export::fmt::Debug
{
    println!("Before serialize, on the emit side : {:?}", t);
    let serialized : String = serde_json::to_string(&t).unwrap();
    stream.write(&serialized.as_bytes());
}

pub fn receive(listener: TcpListener) -> Option<CmdArgs> {
    let mut serialized = String::new();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                match stream.read_to_string(&mut serialized) {
                    Ok(_) => {;},
                    Err(e) => {println!("{:?}", e); continue ;},
                }
                match stream.flush() {
                    Ok(_) => {;},
                    Err(e) => {println!("{:?}", e); continue ;},
                }

                let deserialized: CmdArgs = serde_json::from_str(&serialized).unwrap();
                println!("After deserialize : {:?}", deserialized);
                return (Some(deserialized));
            }
            Err(e) => {
                println!("error : {:?}", e);
                return (None);
            }
        }
    }
    return (None);
}

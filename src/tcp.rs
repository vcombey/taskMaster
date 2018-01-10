use serde;

mod ressources {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct Foo {
        pub string: String,
        pub int: i32,
        pub vec: Vec<String>,
    }
}

use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read};
use serde::{Serialize, Serializer};
use serde_json;
use cmd::Cmd;

#[test]
fn test() {
    let listening_stream = TcpListener::bind("127.0.0.1:4242")
        .unwrap();
    let emit_stream = TcpStream::connect("127.0.0.1:4242")
        .unwrap();

    emit(emit_stream);
    receive(listening_stream);
}

pub fn emit(mut stream: TcpStream) {
    // Creating a struct;
    let foo = ressources::Foo {
        string: String::from("This is a string"),
        int: 8,
        vec: vec![String::from("String in a vec")],
    };

    // Opening a stream;

    println!("Before serialize, on the emit side : {:?}", foo);
    let serialized : String = serde_json::to_string(&foo).unwrap();
    stream.write(&serialized.as_bytes());
}

pub fn receive(listener: TcpListener) {
    let mut serialized = String::new();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                //let (mut stream, _) = listener.accept().unwrap();

                match stream.read_to_string(&mut serialized)
                stream.flush().unwrap();

                let deserialized: ressources::Foo = serde_json::from_str(&serialized).unwrap();
                println!("After deserialize : {:?}", deserialized);
            }
            Err(e) => {
                println!("error : {:?}", e);
            }
        }
    }
}

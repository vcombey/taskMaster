use std::net::{TcpStream,TcpListener};
use std::io::{Write, Read};

use serde::{Serialize, Serializer};
use serde_json;
use cmd::Cmd;

fn emit<T>(stream: &mut TcpStream, t: T)
    where T: serde::Serialize + serde::export::fmt::Debug
{
    println!("Before serialize, on the emit side : {:?}", t);
    let serialized : String = serde_json::to_string(&t).unwrap();
    stream.write(&serialized.as_bytes());
}

/*impl TcpTwoSide {
    pub fn new(bind: &str, connect: &str) -> TcpTwoSide {
        TcpTwoSide {
            listener: {
                match TcpListener::bind(bind) {
                    Ok(listener) => listener,
                    Err(e) => {eprintln!("{:?}", e); None},
                }
            },
            emiter: {
                match TcpStream::connect(connect) {
                    Ok(emiter) => emiter,
                    Err(e) => {eprintln!("{:?}", e); None},
                }
            },
        }
    }
    pub fn emit<T>(&mut self, t: T)
        where T: serde::Serialize + serde::export::fmt::Debug
        {
            println!("Before serialize, on the emit side : {:?}", t);
            let serialized : String = serde_json::to_string(&t).unwrap();
            self.emiter.write(&serialized.as_bytes());
        }
}


}
*/

fn main() {
  let mut buffer = String::new();
  let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
  let _ = stream.write(b"lalala");
    let _ = stream.read_to_string(&mut buffer);
    println!("message get{}", buffer);
}

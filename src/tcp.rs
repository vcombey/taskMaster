use serde;

use std::net::{TcpListener, TcpStream};
use std::io::{Write, Read};
use serde::{Serialize, Serializer};
use serde_json;
use cmd::Cmd;
use cli::CmdArgs;

pub struct TcpTwoSide {
    listener_port: String,
    emiter_port: String,
    listener: Option<TcpListener>,
    emiter: Option<TcpStream>,
}

impl TcpTwoSide {
    pub fn try_bind(&mut self) -> &mut TcpTwoSide {
        if self.listener.is_some() {
            return self;
        }
        self.listener = match TcpListener::bind(&self.listener_port) {
            Ok(listener) =>{
                println!("connection etablished on {}",self.listener_port);
                Some(listener)
            },
            Err(e) => {eprintln!("{:?}", e); None},
        };
        self
    }
    pub fn try_connect(&mut self) -> &mut TcpTwoSide{
        if self.emiter.is_some() {
            return self;
        }
        self.emiter = match TcpStream::connect(&self.emiter_port) {
            Ok(emiter) =>{
                println!("connection etablished on {}",self.emiter_port);
                Some(emiter)
            },
            Err(e) => {eprintln!("{:?}", e); None},
        };
        self
    }
    pub fn new(listener_port: &str, emiter_port: &str) -> TcpTwoSide {
        let mut new = TcpTwoSide {
            listener: None,
            emiter: None,
            listener_port: String::from(listener_port),
            emiter_port: String::from(emiter_port),
        };
        new.try_bind().try_connect();
        new
    }
    pub fn emit<T>(&mut self, t: T)
        where T: serde::Serialize + serde::export::fmt::Debug
        {
            self.try_connect();
            if self.emiter.is_none() {
                return ;
            }
            println!("Before serialize, on the emit side : {:?}", t);
            let serialized : String = serde_json::to_string(&t).unwrap();
            if let Some(ref mut emiter) = self.emiter {
                emiter.write(&serialized.as_bytes());
                emiter.flush();
            }
        }
    pub fn receive<T>(&mut self) -> Option<T>
        where T: for<'de> serde::Deserialize<'de> + serde::export::fmt::Debug
        {
            self.try_bind();
            if self.listener.is_none() {
                return None;
            }
            let mut serialized = String::new();
            match self.listener {
                Some(ref listener) => {
                    for stream in listener.incoming() {
                        match stream {
                            Ok(mut stream) => {
                                /*
                                match stream.read_to_string(&mut serialized) {
                                    Ok(_) => {;},
                                    Err(e) => {println!("{:?}", e); continue ;},
                                }
                                match stream.flush() {
                                    Ok(_) => {;},
                                    Err(e) => {println!("{:?}", e); continue ;},
                                }
                                println!("receive {}", serialized); 
*/
                                return match serde_json::from_reader(stream) {
                                    Ok(t) => Some(t),
                                    Err(e) => {
                                        println!("{:?}", e);
                                        None
                                    },
                                }
                            }
                            Err(e) => {
                                println!("error : {:?}", e);
                                return None;
                            }
                        }
                    }
                    return None;
                }
                None => None,
            }
        }
}

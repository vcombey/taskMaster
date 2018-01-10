// Loading YAML
extern crate yaml_rust;
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};

// Loading std
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

// Sourcing submodules
pub mod service;
pub mod config;
pub mod cmd;

// Loading submodules's struct
use self::service::Service;
use self::config::Config;

pub struct TmStruct<'tm, 'sv, 'c> {
    config_file: &'tm str,
    service_hash: HashMap<&'sv str, service::Service<'sv, 'c>>,
}

impl<'tm, 'sv, 'c> TmStruct<'tm, 'sv, 'c> {
    pub fn new(config_file: &'tm str) -> TmStruct<'tm, 'sv, 'c> {
        TmStruct {
            config_file,
            service_hash: HashMap::new(),
        }
    }

    pub fn parse_config_file(&'tm self) -> Result<Vec<Yaml>, String>{
        /// Reads the content of the config file, and transforms it into a vector of Yaml struct.
        let mut stream = match File::open(self.config_file) {
            Ok(stream) => stream,
            Err(_) => return Err(String::from("An error happened when opening the config file")),
        };

        let mut content = String::new();
        if let Err(_) = stream.read_to_string(&mut content) {
            return Err(String::from("An error happened when reading the content of config file"))
        }
        match YamlLoader::load_from_str(&content) {
            Err(_) => return Err(String::from("An error happened when converting to YAML struct")),
            Ok(yaml) => Ok(yaml),
        }
    }

    pub fn launch_from_hash(& mut self, map: HashMap<String, HashMap<String, Config>>) {
        for (service, map) in map.iter() {
            let mut s = Service::new(service);
            s.launch_from_hash(map);
           	self.service_hash.insert(&service, s);
        }
    }
}

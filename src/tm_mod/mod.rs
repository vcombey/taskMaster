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

pub struct TmStruct<'tm> {
    config_file: &'tm str,
    service_hash: HashMap<String, service::Service>,
}

impl<'tm> TmStruct<'tm> {
    pub fn new(config_file: &'tm str) -> TmStruct<'tm> {
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
        for (service, map) in map.into_iter() {
            let mut s = Service::new(service);
            s.launch_from_hash(map);
            self.service_hash.insert(s.name.clone(), s);
        }
    }

    pub fn hash_config(&self) -> HashMap<String, HashMap<String,Config>> {
        /// Reads the config file using TmStruct methods, and turns it
        /// into a HashMap representing the structure of the services and
        /// programm we need to launch. Multiple service cannot have the
        /// same name, and multiple process cannot have the same name EVEN
        /// ACROSS different services, and finally a process cannot have
        /// the same name a service does. 0 ambiguity allowed.
        let doc = task_master.parse_config_file().unwrap();
        let doc = &doc[0];
        let doc = doc.as_hash().unwrap();

        /*pk ? pas check little map */
        let mut taken_process_names: Vec<&str> = Vec::new();


        // Big map build
        let mut big_map = HashMap::new();
        for (section_name_yaml, section_yaml) in doc.iter() {
            let section_name = section_name_yaml.as_str().unwrap();
            let section_hash = section_yaml.as_hash().unwrap();

            // Litle map build
            let mut little_map = HashMap::new();
            for (name, config) in section_hash.iter() {
                match (name.as_str(), config["cmd"].as_str()) {
                    (Some(name), None) => eprintln!("Missing command for process {}", name),
                    (None, Some(_)) => eprintln!("Missing process name"),
                    (None, None) => eprintln!("Missing both process name and command"),
                    (Some(name), Some(argv)) => {

                        //  Check if a service/process with the same name aready exists
                        if big_map.contains_key(name) {
                            eprintln!("Cannot create process of the name '{}': a service of the same name already exists", name);
                            panic!("Need to improve this server.c");
                        } else if taken_process_names.contains(&name) {
                            eprintln!("Cannot create process of the name '{}': a process of the same name already exists", name);
                            panic!("Need to improve this server.c");
                        }

                        // Insert into little map
                        little_map.insert(String::from(name), Config::from_yaml(name, argv, config));
                        taken_process_names.push(name);
                    },
                }

            }
            // Check if a service / process with the same name already exists
            //if let Some(_) = big_map.keys().find(|key| { key == &section_name}) {
            if big_map.contains_key(section_name) {
                eprintln!("Cannot create service of the name '{}': a service of the same name already exists", section_name);
                panic!("Need to improve this server.c");
            } else if taken_process_names.contains(&section_name) {
                eprintln!("Cannot create service of the name '{}': a process of the same name already exists", section_name);
                panic!("Need to improve this server.c");
            }
            // Insert into big map
            big_map.insert(String::from(section_name), little_map);
        }
        return big_map;
    }
}

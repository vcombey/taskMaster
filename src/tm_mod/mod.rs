// Loading YAML
extern crate yaml_rust;
#[allow(unused_imports)]
use yaml_rust::{Yaml,YamlLoader, YamlEmitter};

// Loading std
use std::collections::HashMap;
use std::fs::File;
use std::sync::mpsc;
use std::io::Read;

// Sourcing submodules
pub mod service;
pub mod config;
pub mod cmd;
pub mod exec_error;

// Loading submodules's struct
use self::service::Service;
use self::config::Config;

use tm_mod::cmd::Target;
use tm_mod::cmd::Cmd;
use tm_mod::cmd::Instruction;
use tm_mod::exec_error::ExecErrors;
use tm_mod::exec_error::ExecError;

#[derive(Debug)]
pub struct TmStruct<'tm> {
    config_file: &'tm str,
    service_hash: HashMap<String, service::Service>,
    receiver_from_threads: mpsc::Receiver<String>,
    sender_to_main: mpsc::Sender<String>,
}

impl<'tm> TmStruct<'tm> {
    pub fn new(config_file: &'tm str) -> TmStruct<'tm> {
        let (sender_to_main, receiver_from_threads) = mpsc::channel();
        TmStruct {
            config_file,
            service_hash: HashMap::new(),
            receiver_from_threads,
            sender_to_main,
        }
    }

    fn send_to_process(&self, p_name: &str, ins: Instruction) -> Result<(), ExecErrors> {
        for (_, service) in self.service_hash.iter() {
            if service.contains_process(p_name) {
                return service.send_to_process(p_name, ins);
            }
        }
        ExecErrors::result_from_e_vec(vec![ExecError::ProcessName(String::from(p_name))])
    }
    
    fn send_to_all_service(&self, ins: Instruction) -> Result<(), ExecErrors> {
        let e: Vec<ExecError> = self.service_hash.values()
            .filter_map(|s| s.send_to_all_process(ins).err())
            .flat_map(|e| e.e_vect.into_iter())
            .collect();

        ExecErrors::result_from_e_vec(e)
    }

    fn send_to_service(&self, s_name: &str, ins: Instruction) -> Result<(), ExecErrors> {
        let service = self.service_hash.get(s_name)
            .ok_or(ExecError::ServiceName(String::from(s_name)));

        service.map_err(|e| ExecErrors{e_vect: vec![e]})
            .and_then(|s| s.send_to_all_process(ins))
    }

    fn send_to_service_process(&self, s_name: &str, p_name: &str, ins: Instruction) -> Result<(), ExecErrors> {
        let service = self.service_hash.get(s_name)
            .ok_or(ExecError::ServiceName(String::from(s_name)));

        service.map_err(|e| ExecErrors{e_vect: vec![e]})
            .and_then(|s| s.send_to_process(p_name, ins))
    }

    pub fn exec_cmd(&mut self, cmd: Cmd) -> Result<(), ExecErrors> {
        let ins = cmd.instruction;
        let e: Vec<ExecError>  = cmd.target_vec.into_iter().filter_map(|target| {
            match target {
                Target::ALL => self.send_to_all_service(ins),
                Target::Process(p_name) => self.send_to_process(&p_name, ins),
                Target::Service(s_name) => self.send_to_service(&s_name, ins),
                Target::ServiceProcess((s_name, p_name)) => self.send_to_service_process(&s_name, &p_name, ins),
            }.err()
        }).flat_map(|e| e.e_vect.into_iter())
        .collect();

        ExecErrors::result_from_e_vec(e)//.map_err(|e| println!("error is: {}",e));
    }

    /// Reads the content of the config file, and transforms it into a vector of Yaml struct.
    pub fn parse_config_file(&'tm self) -> Vec<Yaml>{
        let mut stream = File::open(self.config_file)
            .expect("An error happened when opening the config file");

        let mut content = String::new();
        stream.read_to_string(&mut content)
            .expect("An error happened when reading the content of config file");

        return YamlLoader::load_from_str(&content)
            .expect("An error happened when converting to YAML struct");
    }

    pub fn launch_from_hash(&mut self, map: HashMap<String, HashMap<String, Config>>) {
        for (service, map) in map.into_iter() {
            let mut s = Service::new(service);
            s.launch_from_hash(map, &mut self.sender_to_main);
            self.service_hash.insert(s.name.clone(), s);
        }
    }

    /// Reads the config file using TmStruct methods, and turns it
    /// into a HashMap representing the structure of the services and
    /// programm we need to launch. Multiple service cannot have the
    /// same name, and multiple process cannot have the same name EVEN
    /// ACROSS different services, and finally a process cannot have
    /// the same name a service does. 0 ambiguity allowed.
    pub fn hash_config(&self) -> HashMap<String, HashMap<String,Config>> {
        let doc = self.parse_config_file();
        let doc = &doc[0];
        let doc = doc.as_hash().unwrap();

        let mut taken_process_names: Vec<&str> = Vec::new();

        // Big map build
        let mut service_hash = HashMap::new();
        for (service_name, service_yaml) in doc.iter() {
            let service_name = service_name.as_str().unwrap();
            let service_yaml = service_yaml.as_hash().unwrap();

            // Litle map build
            let mut process_map = HashMap::new();
            for (process_name, process_config) in service_yaml.iter() {
                let process_name = process_name.as_str()
                    .expect(&format!("Missing command for process {:?}", process_name));
                let argv = process_config["cmd"].as_str()
                    .expect("Missing process name");

                //  Check if a service/process with the same name aready exists
                if taken_process_names.contains(&process_name) {
                    panic!("Cannot create process of the name '{}': a process of the same name already exists", process_name);
                }
                // Insert into little map
                process_map.insert(String::from(process_name),
                Config::from_yaml(process_name, argv, process_config));
                taken_process_names.push(process_name);
            }
            // Check if a service / process with the same name already exists
            if service_hash.contains_key(service_name) {
                panic!("Cannot create service of the name '{}': a service of the same name already exists", service_name);
            }
            // Insert into big map
            service_hash.insert(String::from(service_name), process_map);
        }
        return service_hash;
    }
    pub fn receive_from_threads(&self) {
        loop {
            match self.receiver_from_threads.try_recv() {
                Ok(mess) => {
                    eprintln!("message receive {}", mess);
                },
                Err(e) => { eprintln!("{:?}", e); },
            }
        }
    }
}


#[cfg(test)]
mod test {
#[test]
    fn test_bad_file() {
    }
#[test]
    fn test_bad_yaml() {
    }

}

use super::super::cli;

#[derive(Serialize, PartialEq, Deserialize, Debug)]
pub enum Instruction {
    START,
    RESTART,
    STOP,
    RELOAD,
    STATUS,
    SHUTDOWN,
}

/// List of possible targets for an instruction.
/// ALL -> Every single process in every service.
/// Process(p) -> The process with name p.
/// Service(s) -> Every single process in service named s.
/// ServiceProcess(s, p) -> The process name p in service s.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Target {
    ALL,
    Process(String),
    Service(String),
    ServiceProcess((String, String)),
}

/// The struct that will be sent to the server, representing the
/// operation to launch and the targets to launch it onto.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Cmd {
    instruction: Instruction,
    target_vec: Vec<Target>,
}

impl Cmd {
    /// Creates an instance of the cmd struct with already complete values.
    pub fn new(instruction: Instruction, target_vec: Vec<Target>) -> Cmd {
        Cmd {
            instruction,
            target_vec,
        }
    }

    /// Create a command from a vector of string.
    pub fn from_vec(word_list: Vec<&str>) -> Result<Cmd, String> {
        let mut instruction = Instruction::SHUTDOWN; // Default init. Add an empty state to the enum ?
        if let Some(value) = word_list.get(0) {
			instruction = match *value {
                "start" => Instruction::START,
                "restart" => Instruction::RESTART,
                "stop" => Instruction::STOP,
                "reload" => Instruction::RELOAD,
                "status" => Instruction::STATUS,
                "shutdown" => Instruction::SHUTDOWN,
                value => return Err(format!("Invalid command '{}'\n{}", value, cli::HELP_DISPLAY)),
            }
        }
        let mut target_vec: Vec<Target> = Vec::new();
        if instruction != Instruction::SHUTDOWN {
            if let Some(target_slice) = word_list.get(1..) {
                for target in target_slice.iter() {
                    let ret = Target::from_str(*target)?;
                    target_vec.push(ret);
                }
            } else {
                return Err("Missing target".to_string());
            }
            // Some(["", ..]) => return Err(format!("Missing target")),
        }
        Ok(Cmd {instruction, target_vec,})
    }
}

impl Target {
    /// Receives a string that represents a target, returns the
    /// correct Target enum that fits the pattern.
    pub fn from_str(chunk: &str) -> Result<Target, String> {
        if chunk.contains(":") { // The string is potentially under the form service:process
            // Extract the service, and then the process.
            let mini_vec: Vec<&str>= chunk.split(":").collect();

            // Service
            let service = match mini_vec.get(0) {
                Some(service) => service,
                None => return Err(format!("Missing service name. Type 'help' to see different commands and syntaxs")),
            };

			// Process
            let process = match mini_vec.get(1) {
                Some(process_name) => process_name,
                None => return Err("Missing process name. Type 'help' to see different commands and syntaxs".to_string()),
            };

            // Making sure there isnt an invalid syntax like service:process:invalid
            if let Some(val) = mini_vec.get(2) {                 if *val != "" {
                    return Err("Process is the bottom level of the hierarchy: Do not add ':' after a process".to_string());
                }
            }

			// Retrieving exact pattern of target
            match (service, process) {
                (service_name, &"") => Err("Missing process name. Type 'help' to see different commands and syntaxs".to_string()), // No process name
                (service_name, &"*") => Ok(Target::Service(service_name.to_string())), // ALL with *
                (service_name, process_name) => Ok(Target::ServiceProcess((service_name.to_string(), process_name.to_string()))), // Expected service:process
            }

            // End of service:process
        } else { 	// Single word pattern, either all or process_name
            match chunk {
                "all" | "ALL" | "All" => Ok(Target::ALL),
                process_name => Ok(Target::Process(process_name.to_string())),
            }
        }
    }
}

#[cfg(test)]
pub mod test_cmd {
    use super::*;

    #[test]
    fn test_eq_instruction_r() {
        assert_eq!(Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::ALL],
        },
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::ALL],
                   });
    }

    #[test]
    fn test_ne_instruction_w() { 
        assert_ne!(Cmd {
            instruction: Instruction::SHUTDOWN,
            target_vec: vec![Target::ALL],
        },
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::ALL],
                   });
    }

    #[test]
    fn test_eq_targets_r() {
        assert_eq!(Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::Process(String::from("Lorem ipsum"))],
        },
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::Process(String::from("Lorem ipsum"))],
                   });
    }

    #[test]
    fn test_ne_targets_w() {
        assert_ne!(Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::Process(String::from("Lorem ipsum"))],
        },
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::Service(String::from("Lorem ipsum"))],
                   });
    }

    #[test]
    fn test_ne_targets_diff_str_w() {
        assert_ne!(Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::Service(String::from("Lorem"))],
        },
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::Service(String::from("Lorem ipsum"))],
                   });
    }

    #[test]
    fn test_cmd_simple_process_r() {
        assert_eq!(Cmd::from_vec(vec!["start", "process_name"]).unwrap(),
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::Process("process_name".to_string())],
                   });
    }

    #[test]
    fn test_cmd_simple_process_w() {
        assert_ne!(Cmd::from_vec(vec!["start", "process_name"]).unwrap(),
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::Service("process_name".to_string())],
                   });
    }

    #[test]
    fn test_cmd_no_process_name() {
        println!("{:?}", Cmd::from_vec(vec!["start", "service_name:"]));
        assert_eq!(Cmd::from_vec(vec!["start", "service_name:"]), Err("Missing process name. Type 'help' to see different commands and syntaxs".to_string()));
    }

    #[test]
    fn test_cmd_simple_service_process() {
        assert_eq!(Cmd::from_vec(vec!["start","service_name:process_name"]).unwrap(),
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::ServiceProcess(("service_name".to_string() ,"process_name".to_string()))],
                   });
    }

    #[test]
    fn test_cmd_many_process() {
        assert_eq!(Cmd::from_vec(vec!["start","process_one", "process_two"]).unwrap(),
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::Process("process_one".to_string()), Target::Process("process_two".to_string())],
                   });
    }

    #[test]
    fn test_cmd_all() {
        assert_eq!(Cmd::from_vec(vec!["start","all"]).unwrap(),
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::ALL],
                   });
    }

    #[test]
    fn test_cmd_too_many_level() {
        assert_eq!(Cmd::from_vec(vec!["start","1:2:3"]), Err("Process is the bottom level of the hierarchy: Do not add ':' after a process".to_string()));
    }

    #[test]
    fn test_cmd_mix() {
        eprintln!("{:?}", Cmd::from_vec(vec!["start","process_one", "service_one:process_two"]));
        assert_eq!(Cmd::from_vec(vec!["start","process_one", "service_one:process_two"]).unwrap(),
                   Cmd {
                       instruction: Instruction::START,
                       target_vec: vec![Target::Process("process_one".to_string()), Target::ServiceProcess(("service_one".to_string(), "process_two".to_string()))],
                   });
    }
    #[test]
    fn invalid_dsemi() {
        assert_eq!(Cmd::from_vec(vec!["start", "s1::p2"]), Err("Process is the bottom level of the hierarchy: Do not add ':' after a process".to_string()));
    }
}



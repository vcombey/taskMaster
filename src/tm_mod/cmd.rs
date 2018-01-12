use super::super::cli;
use std::str::Split;
use std::str::pattern::Pattern;

#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub enum Target {
    ALL,
    Process(String),
    Service(String),
    ServiceProcess((String, String)),
}

/// The struct that will be sent to the server, representing the
/// operation to launch and the targets to launch it onto.
#[derive(Serialize, Deserialize, Debug)]
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

    /// Create a command struct from the string representing the
    /// instruction and the an iterator to retrieve the targets
    pub fn from_iterator<'a, P>(cmd: &str, it: Split<'a, P>) -> Result<Cmd, String>
        where P: Pattern<'a>,
    {
        let instruction = match cmd {
            "start" => Instruction::START,
            "restart" => Instruction::RESTART,
            "stop" => Instruction::STOP,
            "reload" => Instruction::RELOAD,
            "status" => Instruction::STATUS,
            "shutdown" => Instruction::SHUTDOWN,
            value => return Err(format!("Unvalid command\n{}", cli::HELP_DISPLAY)),
        };

        let mut target_vec = Vec::new();
        for target in it {
            target_vec.push(Target::from_str(target).unwrap());
        }
        Ok(Cmd {
            instruction,
            target_vec,
        })
    }
}

impl Target {
    /// Receives a string that represents a target, returns the
    /// correct Target enum that fits the pattern.
    pub fn from_str(chunk: &str) -> Result<Target, String> {
        if chunk.contains(":") { // The string is potentially under the form service:process
            // Extract the service, and then the process.
            let mini_vec: Vec<&str>= chunk.split(":").collect();
            let service = match mini_vec.get(0) { // Service
                Some(service) =>service.to_string(),
                None => return Err(format!("Missing service name. Type \"help\" to see different commands and syntaxs")),
            };
            let process = match mini_vec.get(1) { // Process
                Some(process_name) => process_name.to_string(),
                None => return Err(format!("Missing process name. Type \"help\" to see different commands and syntaxs")),
            };
            if let Some(val) = mini_vec.get(2) { // Making sure there isnt an invalid syntax like service:process:invalid
                if *val != "" {
                    return Err(format!("Process is the bottom level of the hierarchy: Do not add ':' after a process"));
                }
            }
            // INFECTE
            let str1 = String::from("");
            let str2 = String::from("*");

            match (service, process) {
                (service_name, str1) => Err(format!("Missing process name. Type \"help\" to see different commands and syntaxs")), // No process name
                (service_name, str2) => Ok(Target::Service(service_name)), // ALL with *
                (service_name, process_name) => Ok(Target::ServiceProcess((service_name, process_name))),
            }
        } else {
            match chunk {
                "all" | "ALL" | "All" => Ok(Target::ALL),
                process_name => Ok(Target::Process(process_name.to_string())),
            }
        }
    }
}

use super::super::cli;

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
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
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub enum Target {
    ALL,
    Process(String),
    Service(String),
    ServiceProcess((String, String)),
}

/// The struct that will be sent to the server, representing the
/// operation to launch and the targets to launch it onto.
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct Cmd {
    instruction: Instruction,
    target_vec: Vec<Target>,
}

impl Cmd {
    pub fn new(instruction: Instruction, target_vec: Vec<Target>) -> Cmd {
        Cmd {
            instruction,
            target_vec,
        }
    }
    /// Create a command struct from the string representing the
    /// instruction and the an iterator to retrieve the targets
    pub fn from_line(line: &str) -> Result<Cmd, String>
    {
        let mut split = line.split(" ");
        let instruction = match split.next() {
            Some("start") => Instruction::START,
            Some("restart") => Instruction::RESTART,
            Some("stop") => Instruction::STOP,
            Some("reload") => Instruction::RELOAD,
            Some("status") => Instruction::STATUS,
            Some("shutdown") => Instruction::SHUTDOWN,
            value => return Err(format!("Unvalid command\n{}", cli::HELP_DISPLAY)),
        };

        let mut target_vec = Vec::new();
        for target in split {
            let target = Target::from_str(target)?;
            target_vec.push(target);
        }
        Ok(Cmd {
            instruction,
            target_vec,
        })
    }
}

const HELP_ERROR: &'static str = "Type \"help\" to see different commands and syntaxs";

impl Target {
    /// Receives a string that represents a target, returns the
    /// correct Target enum that fits the pattern.
    pub fn from_str(chunk: &str) -> Result<Target, String> {
        if chunk.contains(":") { // The string is potentially under the form service:process
            // Extract the service, and then the process.
            let chunk_split: Vec<&str>= chunk.split(":").collect();
            let service: &str = match chunk_split.get(0) { // Service
                Some(service) => service,
                None => return Err(format!("Missing service name. {}", HELP_ERROR)),
            };
            let process: &str = match chunk_split.get(1) { // Process
                Some(process_name) => process_name,
                None => return Err(format!("Missing process name. {}", HELP_ERROR)),
            };
            // Making sure there isnt an invalid syntax like service:process:invalid
            if let Some(val) = chunk_split.get(2) {
                if *val != "" {
                    return Err(format!("Process is the bottom level of \
                                       the hierarchy: Do not add ':' \
                                       after a process"));
                }
            }
            // INFECTE
            match (service, process) {
                (service, "") => Err(format!("Missing process name. {}", HELP_ERROR)),
                (service, "*") => Ok(Target::Service(String::from(service))), // ALL with *
                (service, process) => Ok(Target::ServiceProcess(
                        (String::from(service), String::from(process))
                        )),
            }
        } else {
            match chunk {
                "all" | "ALL" | "All" => Ok(Target::ALL),
                process_name => Ok(Target::Process(String::from(process_name))),
            }
        }
    }
}

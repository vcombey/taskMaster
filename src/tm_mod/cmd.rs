use super::super::cli;

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
    pub fn new(instruction: Instruction, target_vec: Vec<Target>) {
        Cmd {
            instruction,
            target_vec,
        }
    }

    /// Create a command struct from the string representing the
    /// instruction and the an iterator to retrieve the targets
    pub fn from_iterator(cmd: &str, it: Split) -> Result<Cmd, String> {
        let instruction = match cmd {
            "start" => START,
            "restart" => RESTART,
            "stop" => STOP,
            "reload" => RELOAD,
            "status" => STATUS,
            "shutdown" => SHUTDOWN,
            _ => return Err(format!("Unvalid command\n{}", HELP_DISPLAY));
        }

        let mut target_vec = Vec::new();
        for target in it {
        }
    }

}

impl Targets {
    /// Receives a string that represents a target, returns the ///
    /// correct Targets enum that fits the pattern.
    pub fn from_str(chunk: &str) -> Result<Targets, String> {
        if chunk.contains(":") { // The string is potentially under the form service:process
			// Extract the service, and then the process.
            let split = chunk.split(":").collect();
            let service = match split.get(0) {
                Some(service) => service,
                None => return Err(format!("Missing service name. Type \"help\" to see different commands and syntaxs"))
            };
            let process = match split.get([1..]) {
                Some(process) => {
                    if process.contains
                    service
                },
                None => return Err(format!("Missing service name. Type \"help\" to see different commands and syntaxs"))
            };
        } else if {
        } else {
        }
    }
}

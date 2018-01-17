#[derive(Serialize, PartialEq, Deserialize, Clone, Copy, Debug)]
pub enum Instruction {
    START,
    RESTART,
    STOP,
    REREAD,
    STATUS,
    SHUTDOWN,
}

/// List of possible targets for an instruction. /// ALL -> Every single process in every service.
/// Process(p) -> The process with name p.
/// Service(s) -> Every single process in service named s.
/// ServiceProcess(s, p) -> The process name p in service s.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Target {
    ALL,
    Process(String, Option<usize>),
    Service(String),
    ServiceProcess((String, String, Option<usize>)),
}

/// The struct that will be sent to the server, representing the
/// operation to launch and the targets to launch it onto.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Cmd {
    pub instruction: Instruction,
    pub target_vec: Vec<Target>,
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
    pub fn from_vec(word_list: Vec<&str>) -> Result<Cmd, ParseError> {
        let instruction =  match word_list.get(0).unwrap() {
            &"start" => Instruction::START,
            &"restart" => Instruction::RESTART,
            &"stop" => Instruction::STOP,
            &"reread" => Instruction::REREAD,
            &"status" => Instruction::STATUS,
            &"shutdown" => Instruction::SHUTDOWN,
            &value => return Err(ParseError::InvalidCommand(value.to_string())),
        };
        let mut target_vec: Vec<Target> = Vec::new();
        if instruction != Instruction::SHUTDOWN && instruction != Instruction::REREAD {
            if let Some(target_slice) = word_list.get(1..) {
                for target in target_slice.iter() {
                    let ret = Target::from_str(*target)?;
                    target_vec.push(ret);
                }
            } else {
                target_vec.push(Target::ALL);
            }
            if target_vec.is_empty() {
                target_vec.push(Target::ALL)
            }
        }
        Ok(Cmd {instruction, target_vec,})
    }
}

impl Target {
    /// Receives a string that represents a target, returns the
    /// correct Target enum that fits the pattern.
    pub fn from_str(chunk: &str) -> Result<Target, ParseError> {
        let chunk_split: Vec<&str>= chunk.split(":").collect();

        if chunk_split.get(3).is_some() {
            return Err(ParseError::ToManyLevels);
        }
        // Retrieving exact pattern of target
        match (chunk_split.get(0), chunk_split.get(1), chunk_split.get(2)) {
            // All
            (Some(all), None, None) if all.to_lowercase() == "all" => Ok(Target::ALL),
            // process
            (Some(process_name), None, None) => Ok(Target::Process(process_name.to_string(), None)),
            // missing service:
            (Some(&""), _, _) => Err(ParseError::MissingService),
            // missing process:
            (_, Some(&""), _) => Err(ParseError::MissingProcess),
            // service:*
            (Some(service_name), Some(&"*"), _) => Ok(Target::Service(service_name.to_string())), 
            // service:process || process:thread_id
            (Some(name1), Some(name2), None) => match usize::from_str_radix(name2, 10) {
                Ok(id) => Ok(Target::Process(name1.to_string(), Some(id))),
                Err(_) => Ok(Target::ServiceProcess((name1.to_string(), name2.to_string(), None))),
            },
            // service:process:thread_id
            (Some(service_name), Some(process_name), Some(thread_id)) => match usize::from_str_radix(thread_id, 10) {
                Ok(id) => Ok(Target::ServiceProcess((service_name.to_string(), process_name.to_string(), Some(id)))),
                Err(_) => Err(ParseError::BadThreadId),
            },
            _ => Err(ParseError::UnexpectedError),
        } 
    }
}

use std::fmt;
use std::error;
use std::error::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ParseError {
    MissingProcess,
    MissingService,
    BadThreadId,
    MissingTarget,
    UnexpectedError,
    ToManyLevels,
    InvalidCommand(String),
}

impl error::Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseError::MissingProcess => 
                "Missing process name. Type 'help' to see different commands and syntaxs",
            ParseError::MissingService =>
                "Missing service name. Type 'help' to see different commands and syntaxs",
            ParseError::BadThreadId => 
                "bad parsing for thread id : must be a valid usize",
            ParseError::UnexpectedError => "unexpected error",
            ParseError::MissingTarget => "Missing target",
            ParseError::ToManyLevels => "To many levels",
            ParseError::InvalidCommand(_) => "Invalid command",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::MissingProcess |
            ParseError::MissingService |
            ParseError::BadThreadId |
            ParseError::MissingTarget |
            ParseError::ToManyLevels |
            ParseError::UnexpectedError => write!(f, "{}", self.description()),
            ParseError::InvalidCommand(ref name) => write!(f, "{} {}", self.description(), name),
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
            target_vec: vec![Target::Process(String::from("Lorem ipsum"), None)],
        },
        Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::Process(String::from("Lorem ipsum"), None)],
        });
    }

    #[test]
    fn test_ne_targets_w() {
        assert_ne!(Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::Process(String::from("Lorem ipsum"), None)],
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
            target_vec: vec![Target::Process("process_name".to_string(), None)],
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
        assert_eq!(Cmd::from_vec(vec!["start", "service_name:"]), Err(ParseError::MissingProcess));
    }

    #[test]
    fn test_cmd_simple_service_process() {
        assert_eq!(Cmd::from_vec(vec!["start","service_name:process_name"]).unwrap(),
        Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::ServiceProcess(("service_name".to_string() ,"process_name".to_string(), None))],
        });
    }

    #[test]
    fn test_cmd_many_process() {
        assert_eq!(Cmd::from_vec(vec!["start","process_one", "process_two"]).unwrap(),
        Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::Process("process_one".to_string(), None), Target::Process("process_two".to_string(), None)],
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
        assert_eq!(Cmd::from_vec(vec!["start","1:2:3:4"]), Err(ParseError::ToManyLevels));
    }

    #[test]
    fn test_cmd_mix() {
        eprintln!("{:?}", Cmd::from_vec(vec!["start","process_one", "service_one:process_two"]));
        assert_eq!(Cmd::from_vec(vec!["start","process_one", "service_one:process_two"]).unwrap(),
        Cmd {
            instruction: Instruction::START,
            target_vec: vec![Target::Process("process_one".to_string(), None), Target::ServiceProcess(("service_one".to_string(), "process_two".to_string(), None))],
        });
    }
    #[test]
    fn invalid_dsemi() {
        assert_eq!(Cmd::from_vec(vec!["start", "s1::p2"]), Err(ParseError::MissingProcess));
    }
}

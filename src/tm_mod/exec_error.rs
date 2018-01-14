use std::fmt::*;
use std::error;
use std::error::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ExecError {
    /// No process with that name
    ProcessName(String),
    /// No service with that name
    ServiceName(String),
    /// Sending to thread error
    Sending((String, usize)),
}

impl error::Error for ExecError {
    fn description(&self) -> &str {
        match *self {
            ExecError::ProcessName(_) => "no process with name",
            ExecError::ServiceName(_) => "no service with name",
            ExecError::Sending(_) => 
                "problem while sending to to thread",
        }
    }
}

impl Display for ExecError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match *self {
            ExecError::ProcessName(ref name) => write!(f, "{} {}", self.description(), name),
            ExecError::ServiceName(ref name) => write!(f, "{} {}", self.description(), name),
            ExecError::Sending((ref p_name, thread_id)) =>
                write!(f, "{} {} {}", self.description(), p_name, thread_id),
        }
        
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ExecErrors {
    pub e_vect: Vec<ExecError>,
}

impl error::Error for ExecErrors {
    fn description(&self) -> &str {
        "exec errors"
    }
}

impl Display for ExecErrors {
    fn fmt(&self, f: &mut Formatter) -> Result {
        //&self.e_vect.iter().map(|x| x.__description()).collect().join("")
        let message = self.e_vect.iter().fold(String::new(), |acc, x| format!("{}{}", acc, x));
        write!(f, "{}", message)
    }
}

use std::fmt;
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

impl fmt::Display for ExecError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl fmt::Display for ExecErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        //&self.e_vect.iter().map(|x| x.__description()).collect().join("")
        let message = self.e_vect.iter().fold(String::new(), |acc, x| format!("{}{}", acc, x));
        write!(f, "{}", message)
    }
}

impl ExecErrors {
    pub fn result_from_e_vec(e: Vec<ExecError>) -> Result<(), ExecErrors> {
        if e.is_empty() {
            return Ok(());
        }
        else {
            return Err(ExecErrors{e_vect: e});
        }
    }
}

use std::fmt::*;
use std::error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum ExecError {
    /// No process with that name
    ProcessName(String),
    /// No service with that name
    ServiceName(String),
    /// Sending to thread error
    Sending((String, String, i32)),
}


impl ExecError {
    fn __description(&self) -> &str {
        match *self {
            ExecError::ProcessName(name) => &format!("no process with name {}", name),
            ExecError::ServiceName(name) => &format!("no service with name {}", name),
            ExecError::Sending((s_name, p_name, thread_id)) => 
                &format!("problem while sending to {}:{} to thread id {}", s_name, p_name, thread_id),
        }
    }
}

impl Display for ExecError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.__description())
    }
}

impl error::Error for ExecError {
    fn description(&self) -> &str {
        self.__description()
    }
}

impl Display for ExecErrors {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.__description())
    }
}

impl error::Error for ExecErrors {
    fn description(&self) -> &str {
        &self.__description()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct ExecErrors {
    e_vect: Vec<ExecError>,
}

impl ExecErrors {
    fn __description(&self) -> &str {
        &self.e_vect.into_iter().fold(String::new(), |acc, x| format!("{}{}", acc, x.__description()))
        //&self.e_vect.iter().map(|x| x.__description()).collect().join("")
    }
}

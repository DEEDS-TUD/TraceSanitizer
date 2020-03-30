use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct SanError {
    details: String,
}

impl SanError {
    pub fn new(msg: &str) -> SanError {
        SanError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for SanError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for SanError {
    fn description(&self) -> &str {
        &self.details
    }
}

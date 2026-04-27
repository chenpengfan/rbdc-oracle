use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OracleError {
    message: String,
}

impl OracleError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for OracleError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.pad(&self.message)
    }
}

impl StdError for OracleError {}

impl From<oracle::Error> for OracleError {
    fn from(value: oracle::Error) -> Self {
        Self::new(value.to_string())
    }
}

impl From<OracleError> for rbdc::Error {
    fn from(value: OracleError) -> Self {
        Self::from(value.to_string())
    }
}

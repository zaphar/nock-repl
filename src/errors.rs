use std::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::convert::From;

use rustyline::error::ReadlineError;

// TODO(jwall): Maybe this should be done with the macro?
#[derive(Debug)]
pub struct WrappedError {
    msg: String,
    cause: Option<Box<Error>>,
}


impl WrappedError {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        WrappedError {
            msg: msg.into(),
            cause: None,
        }
    }

    pub fn with_cause<S: Into<String>>(msg: S, err: Box<Error>) -> Self {
        WrappedError {
            msg: msg.into(),
            cause: Some(err),
        }
    }
}

impl Display for WrappedError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "WrappedError: {}\n", self.msg));
        if let Some(ref cause) = self.cause {
            try!(write!(f, "Cause:\n\t{}", cause));
        }
        return Ok(());
    }
}

impl Error for WrappedError {
    fn description(&self) -> &str {
        &self.msg
    }

    fn cause(&self) -> Option<&Error> {
        if let Some(ref cause) = self.cause {
            return Some(cause.as_ref());
        }
        return None;
    }
}

impl From<ReadlineError> for WrappedError {
    fn from(err: ReadlineError) -> Self {
        Self::with_cause("Readline Error!", Box::new(err))
    }
}

//! errors implements common error types.
// Copyright (2017) Jeremy A. Wall.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use std::io;
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

impl From<io::Error> for WrappedError {
    fn from(err: io::Error) -> Self {
        Self::with_cause("IO Error!", Box::new(err))
    }
}

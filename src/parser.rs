//! The parser module implements a nock syntax parser.

use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;

use tokenizer::{Tokenizer, TokenizerError, ExpressionReader};


enum Noun {
    Atom(u64),
    Cell(Box<Noun>, Box<Noun>),
}

#[derive(Debug)]
struct ParseError {
    msg: String,
    cause: Option<Box<Error>>,
}


impl ParseError {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        ParseError {
            msg: msg.into(),
            cause: None,
        }
    }

    pub fn new_with_cause<S: Into<String>>(msg: S, err: Box<Error>) -> Self {
        ParseError {
            msg: msg.into(),
            cause: Some(err),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        try!(write!(f, "ParseError: {}\n", self.msg));
        if let Some(ref cause) = self.cause {
            try!(write!(f, "Cause:\n\t{}", cause));
        }
        return Ok(());
    }
}

impl Error for ParseError {
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

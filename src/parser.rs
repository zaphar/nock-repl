//! The parser module implements a nock syntax parser.

use std::str::FromStr;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;

use tokenizer::{Tokenizer, Token, TokenizerError, ExpressionReader};

#[derive(Debug,PartialEq)]
pub enum Noun {
    Atom(u64),
    Cell(Box<Noun>, Vec<Noun>),
}

#[derive(Debug)]
pub struct ParseError {
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

impl From<TokenizerError> for ParseError {
    fn from(err: TokenizerError) -> Self {
        Self::new_with_cause("Tokenizer Error", Box::new(err))
    }
}

pub struct Parser {
    toker: Tokenizer,
}

impl Parser {
    pub fn new(reader: Box<ExpressionReader>) -> Self {
        Parser { toker: Tokenizer::new(reader) }
    }

    fn parse_atom(&mut self, tok: &Token) -> Result<Noun, ParseError> {
        let atom = u64::from_str(&tok.val);
        return match atom {
            Ok(atom) => Ok(Noun::Atom(atom)),
            Err(e) => Err(ParseError::new_with_cause("Atom ParseError", Box::new(e))),
        };
    }

    fn flatten(&self, mut nouns: Vec<Noun>) -> Vec<Noun> {
        if nouns.len() == 1 {
            let noun = nouns.pop();
            if let Some(Noun::Cell(head, tail)) = noun {
                nouns.push(*head);
                let mut new_tail = self.flatten(tail);
                nouns.append(&mut new_tail);
            } else if let Some(noun) = noun {
                nouns.push(noun);
            }
        }
        return nouns;
    }

    fn parse_cell(&mut self) -> Result<Noun, ParseError> {
        let head = try!(self.parse());
        let mut tail = Vec::<Noun>::new();
        loop {
            let tok = try!(self.toker.next());
            if tok.is_atom() {
                tail.push(try!(self.parse_atom(&tok)))
            } else if tok.is_cell_start() {
                tail.push(try!(self.parse_cell()))
            } else if tok.is_cell_end() {
                tail = self.flatten(tail);
                break;
            }
        }
        Ok(Noun::Cell(Box::new(head), tail))
    }

    pub fn parse(&mut self) -> Result<Noun, ParseError> {
        let tok = try!(self.toker.next());
        if tok.is_atom() {
            return self.parse_atom(&tok);
        } else if tok.is_cell_start() {
            return self.parse_cell();
        }
        Err(ParseError::new("Unhandled Token!"))
    }
}

#[cfg(test)]
mod parser_tests {
    use parser::{Parser, Noun};
    use tokenizer::tokenizer_tests::MockReader;

    #[test]
    fn test_parse_simple_atom() {
        let reader = MockReader::new(vec![
            "1".to_string(),
        ]);
        let mut parser = Parser::new(Box::new(reader));
        let noun = parser.parse();
        assert!(noun.is_ok());
        let noun = noun.unwrap();
        assert_eq!(noun, Noun::Atom(1));
    }

    #[test]
    fn test_parse_simple_cell() {
        let reader = MockReader::new(vec![
            "[1 2]".to_string(),
        ]);
        let mut parser = Parser::new(Box::new(reader));
        let noun = parser.parse();
        assert!(noun.is_ok());
        let noun = noun.unwrap();
        assert_eq!(noun,
                   Noun::Cell(Box::new(Noun::Atom(1)), vec![Noun::Atom(2)]));
    }

    #[test]
    fn test_parse_autoconsed_cell() {
        let reader = MockReader::new(vec![
            "[1 2 3]".to_string(),
        ]);
        let mut parser = Parser::new(Box::new(reader));
        let noun = parser.parse();
        assert!(noun.is_ok());
        let noun = noun.unwrap();
        assert_eq!(noun,
                   Noun::Cell(Box::new(Noun::Atom(1)), vec![Noun::Atom(2), Noun::Atom(3)]));
    }

    #[test]
    fn test_parse_consed_cell() {
        let reader = MockReader::new(vec![
            "[1 [2 3]]".to_string(),
        ]);
        let mut parser = Parser::new(Box::new(reader));
        let noun = parser.parse();
        assert!(noun.is_ok());
        let noun = noun.unwrap();
        assert_eq!(noun,
                   Noun::Cell(Box::new(Noun::Atom(1)), vec![Noun::Atom(2), Noun::Atom(3)]));
    }

    #[test]
    fn test_parse_nested_cell() {
        let reader = MockReader::new(vec![
            "[1 [2 3] 4]".to_string(),
        ]);
        let mut parser = Parser::new(Box::new(reader));
        let noun = parser.parse();
        assert!(noun.is_ok());
        let noun = noun.unwrap();
        assert_eq!(noun,
                   Noun::Cell(Box::new(Noun::Atom(1)),
                              vec![Noun::Cell(Box::new(Noun::Atom(2)), vec![Noun::Atom(3)]),
                                   Noun::Atom(4)]));
    }
}

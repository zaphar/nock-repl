//! The parser module implements a nock syntax parser.
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
#![macro_use]

use std::error;
use std::str::FromStr;
use std::fmt;
use std::fmt::{Display, Formatter};

use tokenizer::{Tokenizer, Token, TokenizerError, ExpressionReader};

/// A Noun is an Atom or a Cell.
#[derive(Debug,PartialEq,Clone)]
pub enum Noun {
    Atom(u64),
    Cell(Vec<Noun>),
}

/// atom constructs a Noun::Atom.
pub fn atom(a: u64) -> Noun {
    Noun::Atom(a)
}

impl Display for Noun {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), fmt::Error> {
        match self {
            &Noun::Atom(ref u) => try!(write!(fmt, "{}", u)),
            &Noun::Cell(ref list) => {
                let len = list.len();
                if len > 0 {
                    try!(write!(fmt, "[{}", list[0]));
                }
                if len > 1 {
                    for n in &list[1..] {
                        try!(write!(fmt, " {}", n));
                    }
                }
                try!(write!(fmt, "]"))
            }
        }
        return Ok(());
    }
}

/// cell! constructs a Noun::Cell.
#[macro_export]
macro_rules! cell {    ( $( $x:expr ),* ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            Noun::Cell(Noun::flatten(temp_vec))
        }
    };
}

impl Noun {
    /// flatten flattens a Vec<Nouns> to implement autocons.
    /// [0 [1 [2 3]]] become [0 1 2 3].
    /// but [0 [1 2] 3] stays the same.
    pub fn flatten(mut nouns: Vec<Noun>) -> Vec<Noun> {
        if nouns.len() >= 1 {
            let noun = nouns.pop();
            if let Some(Noun::Cell(mut list)) = noun {
                let head = list.pop().unwrap();
                if list.len() > 0 {
                    nouns.append(&mut list);
                }
                nouns.push(head);
            } else if let Some(noun) = noun {
                nouns.push(noun);
            }
        }
        return nouns;
    }

    /// head returns the head of a Noun::Cell or crashes if it's not a Cell.
    pub fn head(&self) -> Result<&Noun, ParseError> {
        if let &Noun::Cell(ref list) = self {
            if list.len() > 0 {
                return Ok(&list[0]);
            }
        }
        Err(ParseError::new("!! Atoms or ~ have no head"))
    }

    /// tail returns the tail of a Noun::Cell or crashes if it's not a cell.
    pub fn tail(&self) -> Result<&[Noun], ParseError> {
        if let &Noun::Cell(ref list) = self {
            if list.len() > 1 {
                return Ok(&list[1..]);
            }
        }
        Err(ParseError::new("!! Atoms or cells of (len < 2) have no tail"))
    }
}

make_error!(ParseError, "ParseError: {}\n");

// TODO(jeremy): Should this be created by the macro as well?
impl From<TokenizerError> for ParseError {
    fn from(err: TokenizerError) -> Self {
        Self::new_with_cause("Tokenizer Error", Box::new(err))
    }
}

/// Parser parses a Token stream from a Tokenizer into a nock Noun.
pub struct Parser {
    toker: Tokenizer,
}

impl Parser {
    /// Construct a parser from an ExpressionReader.
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

    fn parse_cell(&mut self) -> Result<Noun, ParseError> {
        let mut list = Vec::<Noun>::new();
        list.push(try!(self.parse()));
        loop {
            let tok = try!(self.toker.next());
            if tok.is_atom() {
                list.push(try!(self.parse_atom(&tok)))
            } else if tok.is_cell_start() {
                list.push(try!(self.parse_cell()))
            } else if tok.is_cell_end() {
                list = Noun::flatten(list);
                break;
            }
        }
        Ok(Noun::Cell(list))
    }

    /// Parses a single Noun from the ExpressionReader or returns an error.
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
    use parser::{Parser, Noun, atom};
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
        assert_eq!(noun, Noun::Cell(vec![Noun::Atom(1), Noun::Atom(2)]));
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
                   Noun::Cell(vec![Noun::Atom(1), Noun::Atom(2), Noun::Atom(3)]));
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
                   Noun::Cell(vec![Noun::Atom(1), Noun::Atom(2), Noun::Atom(3)]));
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
                   Noun::Cell(vec![Noun::Atom(1),
                                   Noun::Cell(vec![Noun::Atom(2), Noun::Atom(3)]),
                                   Noun::Atom(4)]));
    }

    #[test]
    fn test_flatten() {
        let reader = MockReader::new(vec![
            "[[1 2] [1 2]]".to_string(),
            "[[1 [2 3] 4] [1 [2 3] 4]]".to_string(),
        ]);
        let mut parser = Parser::new(Box::new(reader));
        let noun = parser.parse();
        assert!(noun.is_ok());
        let noun = noun.unwrap();
        assert_eq!(noun,
                  Noun::Cell(vec![Noun::Cell(vec![atom(1), atom(2)]),
                                  atom(1), atom(2)]));
        let noun = parser.parse();
        assert!(noun.is_ok());
        let noun = noun.unwrap();
        assert_eq!(noun,
                  Noun::Cell(vec![Noun::Cell(vec![atom(1), Noun::Cell(vec![atom(2), atom(3)]), atom(4)]),
                                  atom(1), Noun::Cell(vec![atom(2), atom(3)]), atom(4)]));
    }
}

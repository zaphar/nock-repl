//! The tokenizer module implements a nock tokenizer.
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
use std::error;
use std::fmt;
use std::fmt::Display;
use std::convert::Into;
use std::convert::From;
use std::char;

use errors::WrappedError;

/// Token is a parsed token for a Nock Noun.
/// It includes the line and column that the token was found on.
/// All valid tokens are in the ASCII character set.
#[derive(Debug)]
pub struct Token {
    pub line: usize,
    pub col: usize,
    pub val: String,
}

impl Token {
    pub fn new(c: char, line: usize, col: usize) -> Self {
        Token {
            line: line,
            col: col,
            val: c.to_string(),
        }
    }

    /// append_char appends a char to the value.
    pub fn append_char(&mut self, c: char) {
        self.val.push(c);
    }

    /// is_atom returns true if the token is for a valid atom.
    pub fn is_atom(&self) -> bool {
        self.val.len() > 0 && (self.val.as_bytes()[0] as char).is_digit(10)
    }

    /// is_cell_start returns true if the token is a cell start.
    pub fn is_cell_start(&self) -> bool {
        self.val.len() > 0 && self.val == "["
    }

    /// is_cell_end returns true if the token is the end of a cell.
    pub fn is_cell_end(&self) -> bool {
        self.val.len() > 0 && self.val == "]"
    }
}

make_error!(TokenizerError, "TokenizerError: {}\n");

impl From<WrappedError> for TokenizerError {
    fn from(err: WrappedError) -> Self {
        Self::new_with_cause("Read Error", Box::new(err))
    }
}

/// The ExpressionReader trait represents an interface that will
/// return either a Vec<String> of lines for a valid nock expression.
/// or a WrappedError.
pub trait ExpressionReader {
    fn read(&mut self) -> Result<Vec<String>, WrappedError>;
    // FIXME(jwall): Should this support closing?
}

/// Tokenizer reads a series of tokens from an expression reader.
pub struct Tokenizer {
    curr: Option<Vec<String>>,
    line: usize,
    col: usize,
    reader: Box<ExpressionReader>,
}

impl Tokenizer {
    /// new constructs a Tokenizer from an ExpressionReader.
    pub fn new(reader: Box<ExpressionReader>) -> Self {
        Tokenizer {
            curr: None,
            line: 0,
            col: 0,
            reader: reader,
        }
    }

    /// next returns the next token or a TokenizerError.
    pub fn next(&mut self) -> Result<Token, TokenizerError> {
        try!(self.consume_reader());
        self.get_next_token()
    }

    fn consume_reader(&mut self) -> Result<(), TokenizerError> {
        let mut consume = false;
        if let Some(ref lines) = self.curr {
            if self.line >= lines.len() {
                consume = true;
            }
        } else {
            consume = true;
        }
        if consume {
            self.line = 0;
            self.col = 0;
            self.curr = Some(try!(self.reader.read()));
        }
        Ok(())
    }

    fn get_next_char(&mut self) -> Result<(char, usize, usize), TokenizerError> {
        try!(self.consume_reader());
        if let Some(ref lines) = self.curr {
            // Handle our end of line.
            if self.col >= lines[self.line].len() {
                let (line, col) = (self.line, self.col);
                self.line += 1;
                self.col = 0;
                // We synthesize a newline character to simplify parsing.
                return Ok(('\n', line, col));
            }
            // TODO(jwall): Should we cache this?
            let bytes = &lines[self.line].as_bytes();
            // Since all nock syntax is valid ascii this is a
            // safe cast to do.
            let curr_col = self.col;
            self.col += 1;
            return Ok((bytes[curr_col] as char, self.line, curr_col));
        }
        return Err(TokenizerError::new("End of stream"));
    }

    fn pushback(&mut self, len: usize) {
        // NOTE(jeremy): This is potentially unsafe but since we are in theory
        // only ever pushing back something that we have already consumed in
        // a single line this should be safe.
        self.col -= len;
    }

    fn gobble_atom(&mut self, mut tok: Token) -> Result<Token, TokenizerError> {
        loop {
            // char loop
            let (c, _, _) = match self.get_next_char() {
                Ok(tpl) => tpl,
                Err(_) => return Ok(tok),
            };
            if c.is_whitespace() {
                return Ok(tok);
            }
            if c == '.' {
                // treat . as whitespace inside of an atom.
                // Currently this is pretty dumb and doesn't
                // enforce the right syntax of dotting as comma.
                // i.e. every 3 digits. This is deemed acceptable
                // for now.
                continue;
            }
            if !c.is_digit(10) {
                // Technically this case is an error but we don't emit
                // error tokens here, ever, despite what the type signature
                // states.
                self.pushback(1);
                return Ok(tok);
            }
            tok.append_char(c);
        }
    }

    fn get_next_token(&mut self) -> Result<Token, TokenizerError> {
        loop {
            // char loop
            let (c, line, col) = try!(self.get_next_char());
            match c {
                // open cell
                '[' => {
                    return Ok(Token::new(c, line, col));
                }
                // close cell
                ']' => {
                    return Ok(Token::new(c, line, col));
                }
                // Atom chars
                '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.' => {
                    return self.gobble_atom(Token::new(c, line, col));
                }
                // Whitespace
                ' ' | '\t' | '\n' | '\r' => {
                    // We skip these.
                    continue;
                }
                _ => return Err(TokenizerError::new(format!("Invalid Character: '{}'", c))),
            }
        }
    }
}

#[cfg(test)]
pub mod tokenizer_tests {
    use tokenizer::{ExpressionReader, Tokenizer};
    use errors::WrappedError;

    pub struct MockReader {
        expr: Vec<String>,
        err: bool,
    }

    impl MockReader {
        pub fn new(expr: Vec<String>) -> Self {
            MockReader {
                expr: expr,
                err: false,
            }
        }
    }

    impl ExpressionReader for MockReader {
        fn read(&mut self) -> Result<Vec<String>, WrappedError> {
            if !self.err {
                self.err = true;
                return Ok(self.expr.clone());
            } else {
                return Err(WrappedError::new("End Of Stream"));
            }
        }
    }

    #[test]
    fn mock_reader_sanity_check() {
        let mut reader = MockReader::new(vec![
                "[1 2 3]".to_string(),
            ]);
        let expr = reader.read();
        assert!(expr.is_ok());
        let expr = reader.read();
        assert!(expr.is_err());
    }

    fn assert_token_stream(toker: &mut Tokenizer, expect: Vec<(&str, usize, usize)>) {
        for (v, l, c) in expect {
            let tok = toker.next();
            println!("tok: {:?}", tok);
            assert!(tok.is_ok());
            let tok = tok.unwrap();
            assert_eq!(tok.val, *v);
            assert_eq!(tok.line, l);
            assert_eq!(tok.col, c);
        }
        assert!(toker.next().is_err());
    }

    #[test]
    fn test_tokenizer_simple_one_liner() {
        let reader = MockReader::new(vec![
                "[1 2 3]".to_string(),
            ]);
        let boxed = Box::new(reader);
        let mut toker = Tokenizer::new(boxed);
        let expect = vec![("[", 0, 0), ("1", 0, 1), ("2", 0, 3), ("3", 0, 5), ("]", 0, 6)];
        assert_token_stream(&mut toker, expect);
    }

    #[test]
    fn test_tokenizer_simple_multi_line() {
        let reader = MockReader::new(vec![
                "[1 2 3".to_string(),
                "]".to_string(),
            ]);
        let boxed = Box::new(reader);
        let mut toker = Tokenizer::new(boxed);
        let expect = vec![("[", 0, 0), ("1", 0, 1), ("2", 0, 3), ("3", 0, 5), ("]", 1, 0)];
        assert_token_stream(&mut toker, expect);
    }

    #[test]
    fn test_tokenizer_simple_multi_char_atoms() {
        let reader = MockReader::new(vec![
                "1234567890".to_string(),
                "123  1".to_string(),
            ]);
        let boxed = Box::new(reader);
        let mut toker = Tokenizer::new(boxed);
        let expect = vec![("1234567890", 0, 0),
                          ("123", 1, 0),
                          ("1", 1, 5),
        ];
        assert_token_stream(&mut toker, expect);
    }

    #[test]
    fn test_tokenizer_simple_multi_char_atoms_with_dot() {
        let reader = MockReader::new(vec![
                "123.4567.890".to_string(),
                "123  1".to_string(),
            ]);
        let boxed = Box::new(reader);
        let mut toker = Tokenizer::new(boxed);
        let expect = vec![("1234567890", 0, 0),
                          ("123", 1, 0),
                          ("1", 1, 5),
        ];
        assert_token_stream(&mut toker, expect);
    }
}

//! The tokenizer module implements a nock tokenizer.

use std::io::{Error, ErrorKind};
use std::convert::From;
use std::convert::Into;
use std::char;

#[derive(Debug)]
struct Token {
    line: usize,
    col: usize,
    val: String,
}

impl Token {
    pub fn new(c: char, line: usize, col: usize) -> Self {
        Token {
            line: line,
            col: col,
            val: c.to_string(),
        }
    }

    pub fn append_char(&mut self, c: char) {
        self.val.push(c);
    }
}

#[derive(Debug)]
pub struct TokenizerError {
    msg: String,
    cause: Option<Error>,
}


impl TokenizerError {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        TokenizerError {
            msg: msg.into(),
            cause: None,
        }
    }

    pub fn from_io_error<S: Into<String>>(msg: S, err: Error) -> Self {
        TokenizerError {
            msg: msg.into(),
            cause: Some(err),
        }
    }
}

impl From<Error> for TokenizerError {
    fn from(err: Error) -> Self {
        Self::from_io_error("IO Error", err)
    }
}

// TODO(jwall): Tokenizer error.
pub trait ExpressionReader {
    fn read(&mut self) -> Result<Vec<String>, Error>;
}

pub struct Tokenizer {
    curr: Option<Vec<String>>,
    line: usize,
    col: usize,
    reader: Box<ExpressionReader>,
}

impl Tokenizer {
    pub fn new(reader: Box<ExpressionReader>) -> Self {
        Tokenizer {
            curr: None,
            line: 0,
            col: 0,
            reader: reader,
        }
    }

    pub fn reset_line_col(&mut self) {
        self.line = 0;
        self.col = 0;
    }

    pub fn next(&mut self) -> Result<Token, TokenizerError> {
        if !self.curr.is_some() {
            let expr = try!(self.reader.read());
            self.curr = Some(expr);
        }
        self.get_next_token()
    }

    fn get_next_char(&mut self) -> Result<(char, usize, usize), TokenizerError> {
        if let Some(ref lines) = self.curr {
            if self.col >= lines[self.line].len() {
                let (line, col) = (self.line, self.col);
                self.line += 1;
                self.col = 0;
                // We synthesize a newline character to simplify parsing.
                return Ok(('\n', line, col));
            }
            if self.line >= lines.len() {
                return Err(TokenizerError::new("End of stream"));
            }
            // TODO(jwall): Should we cache this?
            let bytes = &lines[self.line].as_bytes();
            // Since all nock syntax is valid ascii this is a
            // safe cast to do.
            let curr_col = self.col;
            self.col += 1;
            return Ok((bytes[curr_col] as char, self.line, curr_col));
        }
        return Err(TokenizerError::new("Empty Expression!"));
    }

    fn pushback(&mut self, len: usize) {
        // This is potentially unsafe but since we are in theory only
        // ever pushing back something that we have already consumed in
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
        return Ok(tok);
    }

    fn get_next_token(&mut self) -> Result<Token, TokenizerError> {
        let maybe_tok: Option<Token> = None;
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
                    println!("Skipping whitespace char");
                    continue;
                }
                _ => return Err(TokenizerError::new(format!("Invalid Character: '{}'", c))),
            }
        }
    }
}

#[cfg(test)]
mod tokenizer_tests {
    use std::io::{Error, ErrorKind};
    use tokenizer::{ExpressionReader, Tokenizer, TokenizerError};

    struct MockReader {
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
        fn read(&mut self) -> Result<Vec<String>, Error> {
            if !self.err {
                self.err = true;
                return Ok(self.expr.clone());
            } else {
                return Err(Error::new(ErrorKind::Other, "End Of Stream"));
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
        let mut i = 0;
        for (v, l, c) in expect {
            let tok = toker.next();
            println!("tok: {:?}", tok);
            assert!(tok.is_ok());
            let tok = tok.unwrap();
            assert_eq!(tok.line, l);
            assert_eq!(tok.col, c);
            assert_eq!(tok.val, *v);
            i += 1;
        }
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

//! main is our command line application implementation.
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
extern crate clap;
extern crate rustyline;

#[macro_use]
mod macros;

mod tokenizer;
mod parser;
mod errors;
mod nock;

use clap::{App, Arg};
use rustyline::Editor;

use errors::WrappedError;

struct PromptingLineParser {
    read_prompt: String,
    continue_prompt: String,
    is_complete: fn(&Vec<String>) -> bool,
    editor: Editor<()>,
}

impl PromptingLineParser {
    fn new(read_prompt: String,
           continue_prompt: String,
           is_complete: fn(&Vec<String>) -> bool)
           -> PromptingLineParser {
        PromptingLineParser {
            read_prompt: read_prompt,
            continue_prompt: continue_prompt,
            is_complete: is_complete,
            editor: Editor::<()>::new(),
        }
    }
}

impl tokenizer::ExpressionReader for PromptingLineParser {
    fn read(&mut self) -> Result<Vec<String>, WrappedError> {
        let mut buffer = Vec::new();
        let mut prompt = &self.read_prompt;
        loop {
            let line = try!(self.editor.readline(prompt));
            buffer.push(line);
            if (self.is_complete)(&buffer) {
                break;
            } else {
                prompt = &self.continue_prompt;
            }
        }
        self.editor.add_history_entry(&buffer.join("\n"));
        return Ok(buffer);
    }
}

use std::fs::File;
use std::io::BufReader;
use std::io::BufRead;

struct FileExpressionReader {
    name: String,
    buff_reader: Option<BufReader<File>>,
    is_complete: fn(&Vec<String>) -> bool,
    eof: bool,
}

impl FileExpressionReader {
    pub fn new<S: Into<String>>(file_name: S, is_complete: fn(&Vec<String>) -> bool) -> FileExpressionReader {
        FileExpressionReader{name: file_name.into(),
                             buff_reader: None,
                             is_complete: is_complete,
                             eof: false,
        }
    }

    pub fn open(&mut self) -> Result<(), WrappedError> {
        let file = try!(File::open(&self.name));
        self.buff_reader = Some(BufReader::new(file));
        Ok(())
    }
}

impl tokenizer::ExpressionReader for FileExpressionReader {
    fn read(&mut self) -> Result<Vec<String>, WrappedError> {
        if self.eof {
            return Err(WrappedError::new("End of File"));
        }
        let mut buffer = Vec::new();
        loop {
            let mut line = String::new();
            let mut rdr = self.buff_reader.as_mut().expect("");
            let num_read = try!(rdr.read_line(&mut line));
            buffer.push(line);
            if num_read < 1 {
                self.eof = true;
            }
            if (self.is_complete)(&buffer) {
                break;
            }
        }
        return Ok(buffer);
    }
}

fn do_flags<'a>() -> clap::ArgMatches<'a> {
    return App::new("nock")
        .version("0.1")
        .author("Jeremy Wall <jeremy@marzhillstudios.com")
        .about("A simple Nock interpreter and repl")
        .arg(Arg::with_name("file")
            .short("f")
            .long("file")
            .value_name("FILE")
            .help("Execute the nock file.")
            .takes_value(true))
        .get_matches();
}

fn is_complete_expr(lines: &Vec<String>) -> bool {
    let mut count = 0;
    for l in lines {
        for c in l.chars() {
            if c == '[' {
                count += 1;
            }
            if c == ']' {
                count -= 1;
            }
        }
    }
    return count == 0;
}

fn main() {
    let matches = do_flags();
    fn eval_exprs(mut nock_parser: parser::Parser) {
        while let Ok(expr) = nock_parser.parse() {
            match nock::eval(expr) {
                Ok(noun) => println!("{}", noun),
                Err(err) => println!("{}", err),
            }
        }
    }

    if let Some(filename) = matches.value_of("file") {
        // parse and execute file stream.
        let mut reader = FileExpressionReader::new(filename, is_complete_expr);
        reader.open().expect("Failed to open file!");
        let nock_parser = parser::Parser::new(Box::new(reader));
        eval_exprs(nock_parser);
    } else {
        // parse and execute stdin.
        println!("Welcome to the nock repl!");
        println!("Type nock expressions at the prompt.");
        println!("Ctrl-D to quit...\n");
        let reader =
            PromptingLineParser::new("nock> ".to_string(), ">     ".to_string(), is_complete_expr);
        let nock_parser = parser::Parser::new(Box::new(reader));
        eval_exprs(nock_parser);
    }
}

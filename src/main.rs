extern crate clap;
extern crate rustyline;

mod tokenizer;
mod parser;
mod errors;

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
    if let Some(filename) = matches.value_of("file") {
        // parse and execute file stream.
        println!("Executing: {}", filename);
    } else {
        // parse and execute stdin.
        println!("Welcome to the nock repl!");
        println!("Type nock expressions at the prompt.");
        println!("Ctrl-D to quit...\n");
        let reader =
            PromptingLineParser::new("nock> ".to_string(), ">     ".to_string(), is_complete_expr);
        let mut nock_parser = parser::Parser::new(Box::new(reader));
        while let Ok(expr) = nock_parser.parse() {
            println!("Echo: {}", expr)
        }
    }
}

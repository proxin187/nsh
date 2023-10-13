#![warn(clippy::pedantic)]

mod lexer;
mod parser;
mod interpreter;
mod config;
mod escape;

use std::io::Write;
use std::fs;
use std::env;
use std::process;
use argin::Argin;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;


#[derive(Clone, Debug)]
pub enum NshError {
    ReadStdin(String),
    Lexical(String),
    Exec(String),
    ExecWait(String),
    Config(String),
    Prompt(String),
    Parser(String),
    History(String),
    Utf8(String),
}

pub struct NshErrorType {
    errors: Vec<NshError>,
}

impl Default for NshErrorType {
    fn default() -> Self {
        NshErrorType::new()
    }
}

impl NshErrorType {
    pub fn new() -> NshErrorType {
        NshErrorType {
            errors: Vec::new(),
        }
    }

    pub fn push(&mut self, error: NshError) {
        self.errors.push(error);
    }

    pub fn handle_err(&mut self, err: NshError) {
        // handle err is a shortcut for pushing a error and calling handle
        self.push(err);
        self.handle();
    }

    pub fn handle(&mut self) {
        for error in &self.errors {
            match error {
                NshError::ReadStdin(err) => {
                    println!("[ERROR]: Failed to read from `stdin` -> `{err}`");
                },
                NshError::Lexical(err) => {
                    println!("[ERROR]: Lexing with `lib_lexin` failed -> `{err}`");
                },
                NshError::Exec(err) => {
                    println!("[ERROR]: Failed to execute command -> `{err}`");
                },
                NshError::ExecWait(err) => {
                    println!("[ERROR]: Failed to wait for child process -> `{err}`");
                },
                NshError::Config(err) => {
                    println!("[ERROR]: Failed to load config -> `{err}`");
                },
                NshError::Prompt(err) => {
                    println!("[ERROR]: Failed to output prompt -> `{err}`");
                },
                NshError::Parser(err) => {
                    println!("[ERROR]: Failed to parse -> `{err}`");
                },
                NshError::History(err) => {
                    println!("[ERROR]: Failed to load history -> `{err}`");
                },
                NshError::Utf8(err) => {
                    println!("[ERROR]: Failed to parse utf8 -> `{err}`");
                },
            }
        }

        self.errors = Vec::new();
    }

    pub fn merge(&mut self, error: &NshErrorType) {
        self.errors.extend(error.errors.clone());
    }
}


pub struct Nsh {
    errors: NshErrorType,
    vm: interpreter::Machine,
    config: config::Config,
}

impl Nsh {
    pub fn new() -> Nsh {
        Nsh {
            errors: NshErrorType::new(),
            vm: interpreter::Machine::empty(),
            config: config::Config::new(),
        }
    }

    pub fn exec_line(&mut self, buf: &str, output: bool) -> Option<String> {
        // if it matches with a alias then replace it

        let tokens = lexer::tokenize(&buf);

        if let Err(err) = &tokens {
            self.errors.push(NshError::Lexical(err.to_string()));
        }

        // Prevent drop while borrowed in Ast::new()
        let tokens = tokens.unwrap();

        let mut parser = parser::Ast::new(&tokens, &mut self.errors);
        let ast = parser.parse(&self.config);

        if output {
            Some(self.vm.exec(&mut self.config, ast, true).unwrap())
        } else {
            self.vm.exec(&mut self.config, ast, false);
            None
        }
    }

    pub fn load_config(&mut self, file: &str) -> Result<(), Box<dyn std::error::Error>> {
        let buffer = fs::read_to_string(file)?;
        self.exec_line(&buffer, false);
        Ok(())
    }

    fn prompt(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let ps1 = self.config.prompt.clone(); // to prevent immutable borrow inside mutable borrow
        let output = self.exec_line(&ps1, true);
        std::io::stdout().flush()?;
        if let Some(proc_output) = &output {
            Ok(proc_output.clone())
        } else {
            Ok(String::new())
        }
    }
}

fn args() -> Argin {
    let mut arg = Argin::new();
    arg.add_flag("-help");
    arg.parse()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arg = args();
    if arg.flags.contains(&"-help".to_string()) {
        println!("./nsh [OPTIONS]");
        println!("    OPTIONS:");
        println!("        -c <FILE>: load custom config");
        println!("        -help: show this");
        return Ok(());
    }

    // create new nsh instance
    let mut nsh = Nsh::new();
    nsh.vm = interpreter::Machine::new();
    let mut rl = DefaultEditor::new()?;

    // load history and config
    let path = match env::var("HOME") {
        Ok(path) => path,
        Err(err) => {
            println!("[ERROR]: failed to get home directory -> {}", err.to_string());
            println!("[NOTE]: Make sure the $HOME environment variable is set");
            process::exit(1);
        },
    };

    if let Err(err) = rl.load_history(&format!("{path}/.config/nsh/history.txt")) {
        nsh.errors.handle_err(NshError::History(err.to_string()));
    }

    if let Err(err) = nsh.load_config(&format!("{path}/.config/nsh/conf.nsh")) {
        nsh.errors.handle_err(NshError::Config(err.to_string()));
    }

    // Main loop
    loop {
        let prompt = nsh.prompt();
        if let Err(err) = &prompt {
            nsh.errors.push(NshError::Prompt(err.to_string()));
        }

        let readline = rl.readline(&prompt.unwrap());

        match readline {
            Ok(line) => {
                if let Err(err) = rl.add_history_entry(line.as_str()) {
                    nsh.errors.handle_err(NshError::History(err.to_string()));
                }
                nsh.exec_line(&(line + "\n"), false);
            },
            Err(ReadlineError::Interrupted) => {
                // intentionally dont exit when getting ctrl-c
                println!("^C");
            },
            Err(ReadlineError::Eof) => {
                println!("^D");
                return Ok(());
            },
            Err(err) => {
                println!("Unknown err: {}", err.to_string());
                return Ok(());
            },
        }

        // merge the errors from the vm into the main loop
        nsh.errors.merge(nsh.vm.errors());

        nsh.vm.empty_errors();

        nsh.errors.handle();
    }
}



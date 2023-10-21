use crate::parser::{Node, Value, Ast};
use crate::{NshErrorType, NshError};
use crate::escape;
use crate::config::Config;
use crate::lexer;

use std::process::Command;
use std::env;
use lib_lexin::Token;


pub struct Machine {
    errors: NshErrorType,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            errors: NshErrorType::new(),
        }
    }

    pub fn empty_errors(&mut self) {
        self.errors = NshErrorType::new();
    }

    fn value(&self, val: &Value) -> String {
        return match val {
            Value::Str(string) => escape::string(&string),
            Value::Env(var) => {
                let result = env::var(var);
                result.unwrap_or(String::new())
            },
            Value::Nov => String::new(),
        };
    }

    fn arg_values(&self, args: &[Value]) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();

        for val in args {
            result.push(self.value(val));
        }

        result
    }

    fn current_working_dir(&self) -> String {
        let dir = env::current_dir();
        if let Ok(path) = dir {
            return path.to_str().unwrap_or("").to_string();
        } else {
            return String::new();
        }
    }

    fn cd(&mut self, dir: &Value) -> Result<(), NshError> {
        let dir = self.value(dir);
        if let Err(err) = env::set_current_dir(&dir) {
            return Err(NshError::Exec(err.to_string()));
        }

        env::set_var(String::from("PWD"), self.current_working_dir());
        Ok(())
    }

    fn ret_exec(&self, output_on: bool, output: String) -> Option<String> {
        // ret_exec returns either none or a string depending on if the exec functions is called in
        // output mode or not, ret_exec exists to prevent repetitions, when returning
        if output_on {
            return Some(output);
        } else {
            return None;
        }
    }

    fn print_alias(&self, aliases: &Vec<(Token, Vec<Token>)>) {
        for alias in aliases {
            println!("{}: {}", alias.0.as_string(), Ast::tokens_to_string(&alias.1));
        }
    }

    pub fn exec(&mut self, config: &mut Config, ast: &[Node], output_on: bool) -> Option<String> {
        let mut output = String::new();
        for node in ast {
            match node {
                Node::Exec {file, args} => {
                    if output_on {
                        let process = Command::new(file)
                            .args(self.arg_values(args))
                            // here we use output instead of spawn, output will run the program and
                            // collect stdout instead of directly piping the process stdout to the user
                            // stdout
                            .output();

                        if let Err(err) = &process {
                            self.errors.push(NshError::Exec(err.to_string()));
                            return self.ret_exec(output_on, output);
                        } else if let Ok(proc) = &process {
                            let stdout = String::from_utf8(proc.stdout.clone());
                            match stdout {
                                Ok(value) => {
                                    output = output + &value;
                                },
                                Err(err) => {
                                    self.errors.push(NshError::Utf8(err.to_string()));
                                    return self.ret_exec(output_on, output);
                                },
                            }
                        }
                    } else {
                        let process = Command::new(file)
                            .args(self.arg_values(args))
                            // .envs(&self.env)
                            .spawn();

                        if let Err(err) = &process {
                            self.errors.push(NshError::Exec(err.to_string()));
                            return self.ret_exec(output_on, output);
                        }

                        // Calling process::Child::wait() halts execution until the child process has
                        // successfully exited
                        if let Err(err) = process.unwrap().wait() {
                            self.errors.push(NshError::ExecWait(err.to_string()));
                            return self.ret_exec(output_on, output);
                        }
                    }
                },
                Node::Cd(dir) => {
                    if let Err(err) = self.cd(dir) {
                        self.errors.push(err);
                        return self.ret_exec(output_on, output);
                    }
                },
                Node::Alias(original, replacement) => {
                    if *original == Value::default() {
                        // print all the aliases
                        // TODO: append output to output if output is on
                        self.print_alias(&config.alias);
                    } else {
                        // append a alias

                        // signature means a sequence of tokens that can identify the alias
                        let original_signature = lexer::tokenize(&self.value(original));
                        let replacement_signature = lexer::tokenize(&self.value(replacement));

                        if original_signature.is_err() || replacement_signature.is_err() {
                            self.errors.push(NshError::Lexical("failed to tokenize".to_string()));
                            return self.ret_exec(output_on, output);
                        }

                        // to prevent use after move
                        let original_signature = original_signature.unwrap();

                        let original_len = original_signature.len();
                        if original_len != 1 {
                            self.errors.push(NshError::Alias("Alias can only accept 1 token as the match".to_string()));
                            return self.ret_exec(output_on, output);
                        }

                        // unwrap is safe because we checked for errors earlier
                        config.alias.push((original_signature[0].clone(), replacement_signature.unwrap()));
                    }
                },
                Node::SetEnv(env, value) => {
                    env::set_var(env, self.value(value));
                },
                Node::Pipe(_, _) => {
                    // TODO: implement pipe
                },
                Node::Nop => {},
            }
        }

        return self.ret_exec(output_on, output);
    }

    pub fn errors(&self) -> &NshErrorType {
        &self.errors
    }
}



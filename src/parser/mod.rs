use lib_lexin::Token;
use crate::{NshError, NshErrorType};
use crate::config::Config;

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Str(String),
    Env(String),
    // Nov stands for no value
    Nov,
}

impl Default for Value {
    fn default() -> Self { Value::Nov }
}

#[derive(Debug)]
pub enum Node {
    Exec {
        file: String,
        args: Vec<Value>,
    },

    Cd(Value),

    Alias(Value, Value),

    SetEnv(String, Value),

    Pipe(Box<Node>, Box<Node>),

    // Nop stands for no operation
    Nop,
}

impl Default for Node {
    fn default() -> Self { Node::Nop }
}

pub struct Ast<'a> {
    tokens: &'a Vec<Token>,
    errors: &'a mut NshErrorType,
    ast: Vec<Node>,
}

impl<'a> Ast<'a> {
    pub fn new(tokens: &'a Vec<Token>, errors: &'a mut NshErrorType) -> Ast<'a> {
        Ast {
            tokens,
            errors,
            ast: Vec::new(),
        }
    }

    fn value(&mut self, token: &Token) -> Value {
        // identifiers in lib_lexin is anything that is unreconized by the tokenizer
        // therefore when i say ident here it just represents a value

        if let Ok(string) = token.is_ident() {
            return Value::Str(string);
        } else if let Ok(string) = token.is_section("string") {
            return Value::Str(string);
        } else if let Ok(env) = token.is_section("env") {
            return Value::Env(env);
        } else {
            let loc = token.loc();
            self.errors.handle_err(NshError::Parser(format!("{}:{}: expected value but got `{}`", loc.0, loc.1, token.as_string())));
            return Value::default();
        }
    }

    fn parse_node(&mut self, node: &[Token]) -> Option<Node> {
        // parse_node takes a command as argument and parses it into either a command, or builtin
        // operation such as change directory (cd).

        if let Ok(file) = node[0].is_ident() {
            let mut args: Vec<Value> = Vec::new();
            let mut index = 1;
            while index < node.len() {
                let arg = self.value(&node[index]);
                args.push(arg);

                index += 1;
            }

            return Some(Node::Exec {
                file,
                args,
            });
        } else if node[0].is_keyword("cd").is_ok() {
            if node.len() < 2 {
                let loc = node[0].loc();
                self.errors.handle_err(NshError::Parser(format!("{}:{}: expected directory", loc.0, loc.1)));
                println!("[SYNTAX]: cd <Value>");
                return None;
            }

            // using self.value() allows you to cd with any value as directory for example enviroment variables
            let dir = self.value(&node[1]);

            return Some(Node::Cd(dir));
        } else if node[0].is_keyword("alias").is_ok() {
            if node.len() == 1 {
                // empty alias simply prints all the aliases
                return Some(Node::Alias(Value::default(), Value::default()));
            } else if node.len() < 3 {
                let loc = node[0].loc();
                self.errors.handle_err(NshError::Parser(format!("{}:{}: alias expects 2 arguments", loc.0, loc.1)));
                println!("[SYNTAX]: alias <Value> <Value>");
                return None;
            }

            return Some(Node::Alias(self.value(&node[1]), self.value(&node[2])));
        } else if let Ok(env) = node[0].is_section("env") {
            let loc = node[0].loc();

            if node.len() < 3 {
                self.errors.handle_err(NshError::Parser(format!("{}:{}: env expected 2 arguments", loc.0, loc.1)));
            } else if node[1].is_symbol("Equal").is_err() {
                self.errors.handle_err(NshError::Parser(format!("{}:{}: expected `=` but got `{}`", loc.0, loc.1, node[1].as_string())));
            } else {
                // no errors
                return Some(Node::SetEnv(env, self.value(&node[2])));
            }

            // in this case there was an error
            println!("[SYNTAX]: ${}$ = <Value>", env);
            return None;

        } else {
            let loc = node[0].loc();
            self.errors.handle_err(NshError::Parser(format!("{}:{}: expected keyword but got `{}`", loc.0, loc.1, node[0].as_string())));
            return None;
        }
    }

    pub fn tokens_to_string(tokens: &[Token]) -> String {
        let mut string = String::new();
        for token in tokens {
            let token = token.clone();
            string.push(' ');
            string.push_str(&token.as_string());
        }
        string
    }

    fn alias(&self, node: &[Token], config: &Config) -> Vec<Token> {
        // alias simply takes a sequence of tokens and matches it up to each alias and replaces it
        // if it matches else it returns it self

        let mut output: Vec<Token> = node.to_vec();

        for alias in &config.alias {
            if let Some(idx) = output.iter().position(|elem| elem == &alias.0) {
                output.remove(idx);
                for elem in alias.1.iter().rev() {
                    output.insert(idx, elem.clone());
                }
            }
        }
        output
    }

    fn push(&mut self, node: &[Token], config: &Config) {
        if !node.is_empty() {
            // calling alias here allows you to alias in commands with multiple nodes
            let node = self.parse_node(&self.alias(node, config));
            self.ast.push(node.unwrap_or_default());
        }
    }

    pub fn parse(&mut self, config: &Config) -> &[Node] {
        let mut node: Vec<Token> = Vec::new();
        let mut index = 0;

        while index < self.tokens.len() {
            if self.tokens[index].is_symbol("And").is_ok() || self.tokens[index].is_symbol("NewLine").is_ok() {
                self.push(&node, config);
                node = Vec::new();
            } else {
                node.push(self.tokens[index].clone());
            }
            index += 1;
        }

        self.push(&node, config);

        &self.ast
    }
}



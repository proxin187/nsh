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
            self.errors.handle_err(NshError::Parser(format!("{}:{}: expected value but got {:?}", loc.0, loc.1, token)));
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
                self.errors.push(NshError::Parser("expected directory".to_string()));
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
                self.errors.handle_err(NshError::Parser("alias expects 2 arguments".to_string()));
                return None;
            }

            return Some(Node::Alias(self.value(&node[1]), self.value(&node[2])));
        }

        unreachable!();
    }

    fn tokens_to_string(&self, tokens: &[Token]) -> String {
        let mut string = String::new();
        for token in tokens {
            let token = token.clone();
            string.push_str(&token.as_string());
        }
        string
    }

    fn alias(&self, node: &[Token], config: &Config) -> Vec<Token> {
        // alias simply takes a sequence of tokens and matches it up to each alias and replaces it
        // if it matches else it returns it self

        for alias in &config.alias {
            if self.tokens_to_string(&alias.0) == self.tokens_to_string(node) {
                return alias.1.clone();
            }
        }
        node.to_vec()
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



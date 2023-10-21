use lib_lexin::Token;

pub struct Config {
    pub alias: Vec<(Token, Vec<Token>)>,
}


impl Config {
    pub fn new() -> Config {
        Config {
            alias: Vec::new(),
        }
    }
}



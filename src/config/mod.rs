use lib_lexin::Token;

pub struct Config {
    pub prompt: String,
    pub alias: Vec<(Vec<Token>, Vec<Token>)>,
}


impl Config {
    pub fn new() -> Config {
        Config {
            // TODO: implement customization of the prompt trough envionment variables
            prompt: String::from("echo -n \"[\" & echo -n $PWD$ & echo -n \"]# \" & "),
            alias: Vec::new(),
        }
    }
}



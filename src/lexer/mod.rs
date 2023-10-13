use lib_lexin::{Lexer, Section, Token};


pub fn tokenize(source: &str) -> Result<Vec<Token>, Box<dyn std::error::Error>> {
    let mut lexer = Lexer::new(
        &[
            "cd",
            "alias",
        ],
        &[
            Section::new("string", "\"", "\""),
            Section::new("env", "$", "$"),
        ],
        &[
            ('&', "And"),
            ('|', "Or"),
            ('\n', "NewLine"),
        ],
    );
    lexer.load_str(source);

    return lexer.tokenize();
}



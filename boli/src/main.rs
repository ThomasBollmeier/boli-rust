use boli::frontend::lexer::tokens::{Token, TokenType::*};

fn main() {
    let my_first_token = Token {
        token_type: Integer(42),
        line: 1,
        column: 1,
    };
    println!("{:?}", my_first_token);
}

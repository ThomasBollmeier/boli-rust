use boli::frontend::lexer::tokens::Token;

fn main() {
    let my_first_token = Token::new_int(42, 1, 1);
    println!("{:?}", my_first_token);
}

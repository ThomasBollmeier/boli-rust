use boli::frontend::parser::{visitor::JsonData, Parser};

fn main() {
    let parser = Parser::new();
    let code = read_stdin();
    let parse_result = parser.parse(&code);
    if let Ok(ast) = parse_result {
        println!("{}", JsonData::from(ast));
    } else {
        println!("Error: {:?}", parse_result.err().unwrap());
    }
}

fn read_stdin() -> String {
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    buffer
}

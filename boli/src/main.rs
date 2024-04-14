use boli::frontend::parser::{ast::Program, visitor::JsonData, ParseError, Parser as BoliParser};
use clap::Parser;
use std::{
    fs::File,
    io::{stdin, BufReader, Read, Result},
};

#[derive(Debug, Parser)]
#[command(
    author = clap::crate_authors!("\n"),
    version = clap::crate_version!(),
    about = "BOLI - (B)ollmeier's (O)wn (L)isp (I)nterpreter"
)]
struct Options {
    #[arg(help = "Input file to parse", default_value = "-")]
    input_file: String,

    #[arg(short, long, help = "Show the AST after parsing")]
    show_ast: bool,
}

fn main() -> Result<()> {
    let options = Options::parse();

    let code = read_input(options.input_file)?;

    let parser = BoliParser::new();
    let parse_result = parser.parse(&code);

    if options.show_ast {
        show_ast(parse_result);
    }

    Ok(())
}

fn show_ast(parse_result: std::result::Result<Program, ParseError>) {
    if let Ok(ast) = parse_result {
        println!("{}", JsonData::from(ast));
    } else {
        println!("Error: {:?}", parse_result.err().unwrap());
    }
}

fn read_input(file_path: String) -> Result<String> {
    let mut buffer = String::new();

    if file_path == "-" {
        stdin().read_to_string(&mut buffer)?;
    } else {
        let file = File::open(file_path)?;
        let mut reader = BufReader::new(file);
        reader.read_to_string(&mut buffer)?;
    }
    Ok(buffer)
}

use boli::{
    frontend::parser::{json_visitor::JsonData, Parser as BoliParser},
    interpreter::{values::borrow_value, Interpreter},
    repl,
};
use clap::Parser;
use std::{
    fs::File,
    io::{stdin, BufReader, Read, Result},
};

#[derive(Debug, Parser)]
#[command(
    author = clap::crate_authors!("\n"),
    version = clap::crate_version!(),
    about = "BOLI - (B)ollmeier's (O)wn (L)isp (I)mplementation"
)]
struct Options {
    #[arg(help = "Input file or standard input (-)", default_value = "-")]
    input_file: String,

    #[arg(
        short = 'i',
        long = "interactive",
        group = "action",
        help = "start interactive REPL"
    )]
    interactive: bool,

    #[arg(
        short = 'p',
        long = "parse-only",
        group = "action",
        help = "parse and output AST"
    )]
    parse_only: bool,

    #[arg(short = 'r', long = "run", group = "action", help = "run interpreter")]
    run: bool,
}

fn main() -> Result<()> {
    let options = Options::parse();
    let mut code: String = String::new();

    if !options.interactive {
        code = read_input(&options.input_file)?;
    }

    if options.interactive {
        repl::run(&options.input_file)?;
    } else if options.parse_only {
        parse(&code);
    } else {
        interpret(&code);
    }

    Ok(())
}

fn parse(code: &str) {
    let parser = BoliParser::new();
    let parse_result = parser.parse(&code);

    if let Ok(ast) = parse_result {
        println!("{}", JsonData::from(ast));
    } else {
        println!("Error: {:?}", parse_result.err().unwrap());
    }
}

fn interpret(code: &str) {
    let mut interpreter = Interpreter::new();
    let result = interpreter.eval(code);

    match result {
        Ok(value) => println!("{}", borrow_value(&value)),
        Err(err) => println!("Error: {:?}", err),
    }
}

fn read_input(file_path: &str) -> Result<String> {
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

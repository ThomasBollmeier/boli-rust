use boli::{
    frontend::parser::{json_visitor::JsonData, Parser as BoliParser},
    interpreter::{values::borrow_value, Interpreter},
    repl,
};
use clap::{Parser, ValueEnum};
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
    #[arg(value_enum, help = "Action to perform")]
    action: Option<Action>,

    #[arg(help = "Input file or standard input (-)", default_value = "-")]
    input_file: String,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
enum Action {
    /// Parse the input file and show the AST
    Parse,
    /// Parse the input file and run the interpreter
    Run,
    /// Run the REPL
    Repl,
}

fn main() -> Result<()> {
    let options = Options::parse();
    let mut code: String = String::new();

    if let Some(action) = &options.action {
        if action == &Action::Repl {
            if options.input_file != "-" {
                eprintln!("Error: input file is not allowed with REPL");
                std::process::exit(1);
            }
        } else {
            code = read_input(options.input_file)?;
        }
    } else {
        code = read_input(options.input_file)?;
    }

    if let Some(action) = options.action {
        match action {
            Action::Parse => parse(&code),
            Action::Run => interpret(&code),
            Action::Repl => repl::run()?,
        }
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

use boli::{
    frontend::parser::{json_visitor::JsonData, Parser as BoliParser},
    interpreter::{
        environment::Environment,
        misc_functions::{Output, StdOutput},
        module_mgmt::{file_system::new_directory, ModuleDirRef},
        values::borrow_value,
        Interpreter,
    },
    repl,
};
use clap::Parser;
use std::{
    cell::RefCell,
    fs::File,
    io::{stdin, BufReader, Read, Result},
    rc::Rc,
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

    #[arg(
        long = "module-dirs",
        required = false,
        default_value = "",
        help = "colon separated list of directories to search for BOLI modules"
    )]
    module_dirs: String,
}

fn main() -> Result<()> {
    let options = Options::parse();
    let module_dirs = options
        .module_dirs
        .split(':')
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut code: String = String::new();

    if !options.interactive {
        code = read_input(&options.input_file)?;
    }

    if options.interactive {
        repl::run(&options.input_file, &module_dirs)?;
    } else if options.parse_only {
        parse(&code);
    } else {
        interpret(&code, &module_dirs);
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

fn interpret(code: &str, module_dirs: &Vec<String>) {
    let search_dirs: Vec<ModuleDirRef> = if module_dirs.is_empty() {
        vec![new_directory(".", "")]
    } else {
        let mut search_dirs = vec![];
        for path in module_dirs {
            let dir: ModuleDirRef = new_directory(path, "");
            search_dirs.push(Rc::clone(&dir));
        }
        search_dirs
    };
    let output: Rc<RefCell<dyn Output>> = Rc::new(RefCell::new(StdOutput::new()));
    let env = Environment::ref_with_search_dirs_and_output(&search_dirs, &output);
    Environment::read_stdlib(&env);

    let mut interpreter = Interpreter::with_environment(&env);

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

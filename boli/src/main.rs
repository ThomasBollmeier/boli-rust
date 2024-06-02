use boli::{
    frontend::parser::{json_visitor::JsonData, Parser as BoliParser},
    interpreter::{
        environment::EnvironmentBuilder,
        misc_functions::{Output, StdOutput},
        module_mgmt::{file_system::new_directory, ModuleDirRef},
        values::{
            borrow_value, downcast_value, new_valueref, Callable, EvalResult, LambdaValue,
            StrValue, ValueRef, ValueType,
        },
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

    #[arg(trailing_var_arg = true, allow_hyphen_values = true, hide = true)]
    boli_args: Vec<String>,
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
        interpret(&code, &module_dirs, &options.boli_args);
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

fn interpret(code: &str, module_dirs: &Vec<String>, args: &Vec<String>) {
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

    let env = EnvironmentBuilder::new()
        .search_dirs(&search_dirs)
        .output(&output)
        .with_prelude(true)
        .build();

    let mut interpreter = Interpreter::with_environment(&env);

    let result = eval_code(&mut interpreter, code, args);

    match result {
        Ok(value) => println!("{}", borrow_value(&value)),
        Err(err) => println!("Error: {:?}", err),
    }
}

fn eval_code(interpreter: &mut Interpreter, code: &str, main_args: &Vec<String>) -> EvalResult {
    let mut result = interpreter.eval(code);

    if let Ok(_) = result {
        let main_opt = interpreter.env.borrow().get("main");
        if let Some(main) = main_opt {
            if main.borrow().get_type() == ValueType::Lambda {
                let main = &borrow_value(&main);
                let main = downcast_value::<LambdaValue>(main).unwrap();
                let args: Vec<ValueRef> = main_args
                    .iter()
                    .map(|s| {
                        new_valueref(StrValue {
                            value: s.to_string(),
                        })
                    })
                    .collect();
                result = main.call(&args);
            }
        }
    }

    result
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

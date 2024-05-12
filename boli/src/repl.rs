use std::{
    env,
    io::{stdin, stdout, Error, ErrorKind, Result, Write},
    rc::Rc,
};

use crate::interpreter::{
    self,
    environment::{Environment, EnvironmentRef},
    module_mgmt::{file_system::new_directory, module_loader::ModuleLoader, ModuleDirRef},
};

pub fn run(module_file: &str, module_dirs: &Vec<String>) -> Result<()> {
    let env = Environment::new_ref();
    let mut interpreter = interpreter::Interpreter::with_environment(&env);
    if !module_dirs.is_empty() {
        set_module_dirs(&env, &module_dirs);
    } else {
        set_module_dirs(&env, &vec![".".to_string()]);
    }
    Environment::read_stdlib(&env);

    let mut input = String::new();
    let mut line = String::new();
    let mut continued = false;
    let mut result_count = 0;

    print_title();

    if module_file != "-" {
        let module_name = if module_file.ends_with(".boli") {
            module_file[..module_file.len() - 5].to_string()
        } else {
            module_file.to_string()
        };
        let loader = ModuleLoader::new(&env);
        let exported_values = loader.load_module(&module_name).map_err(|err| {
            Error::new(
                ErrorKind::Other,
                format!("Error loading module: {}", err.message),
            )
        })?;
        env.borrow_mut().import_values(exported_values);
    }

    loop {
        if !continued {
            print!("boλi> ");
        } else {
            print!("....> ");
        }
        stdout().flush()?;

        line.clear();
        stdin().read_line(&mut line)?;

        line = line.trim().to_string();

        if line.is_empty() {
            continue;
        }

        if line.starts_with(':') {
            if handle_command(&line) {
                break;
            } else {
                line.clear();
                continue;
            }
        }

        if line.ends_with('\\') {
            line.pop();
            continued = true;
        } else {
            continued = false;
        }

        input.push_str(&line);

        if has_open_parens(&input) {
            input.push_str(" ");
            continued = true;
            continue;
        }

        if continued {
            continue;
        }

        let result = interpreter.eval(input.trim());

        match result {
            Ok(value) => {
                let res = format!("${}", result_count);
                result_count += 1;
                interpreter.set_value(res.clone(), value.clone());
                println!("{} = {}", res, value.borrow());
            }
            Err(e) => eprintln!("{}", e),
        }

        input.clear();
    }

    Ok(())
}

fn set_module_dirs(env: &EnvironmentRef, module_dirs: &Vec<String>) {
    let mut search_dirs = vec![];
    for path in module_dirs {
        let dir: ModuleDirRef = new_directory(path, "");
        search_dirs.push(Rc::clone(&dir));
    }
    Environment::set_module_search_dirs(env, &search_dirs);
}

fn print_title() {
    println!(
        "(B)ollmeier's (O)wn (L)isp (I)mplementation - Version {}",
        env!("CARGO_PKG_VERSION")
    );
    println!("Type :q to quit, :h for help.");
    println!("");
}

fn handle_command(cmd: &str) -> bool {
    match cmd {
        ":q" => true,
        ":h" => {
            print_help();
            false
        }
        _ => {
            eprintln!("Unknown command: {}", cmd);
            false
        }
    }
}

fn print_help() {
    println!(":q - Quit the interpreter");
    println!(":h - Show this help");
}

fn has_open_parens(s: &str) -> bool {
    let mut count = 0;

    for c in s.chars() {
        match c {
            '(' | '[' | '{' => count += 1,
            ')' | ']' | '}' => count -= 1,
            _ => (),
        }
    }

    count > 0
}

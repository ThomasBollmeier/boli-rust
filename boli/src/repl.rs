use std::io::{stdin, stdout, Result, Write};

use crate::interpreter;

pub fn run() -> Result<()> {
    let mut interpreter = interpreter::Interpreter::new();
    let mut input = String::new();
    let mut line = String::new();
    let mut continued = false;
    let mut result_count = 0;

    print_title();

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

        if !is_balanced(&input) {
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

fn is_balanced(s: &str) -> bool {
    let mut count = 0;

    for c in s.chars() {
        match c {
            '(' | '[' | '{' => count += 1,
            ')' | ']' | '}' => count -= 1,
            _ => (),
        }
    }

    count == 0
}

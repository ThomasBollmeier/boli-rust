use std::io::{stdin, stdout, Result, Write};

use crate::interpreter;

pub fn run() -> Result<()> {
    let mut interpreter = interpreter::Interpreter::new();
    let mut input = String::new();
    let mut continued = false;

    print_title();

    loop {
        if !continued {
            print!("boλi> ");
        } else {
            print!("....> ");
        }
        stdout().flush()?;

        stdin().read_line(&mut input)?;

        input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }

        if input.starts_with(':') {
            if handle_command(&input) {
                break;
            } else {
                input.clear();
                continue;
            }
        }

        if input.ends_with('\\') {
            input.pop();
            input.push_str("\n");
            continued = true;
            continue;
        } else {
            continued = false;
        }

        let result = interpreter.eval(input.trim());

        match result {
            Ok(value) => println!("res = {}", value.borrow()),
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

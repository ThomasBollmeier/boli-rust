use std::cell::Ref;

use super::values::*;

pub struct IsEqual {}

impl IsEqual {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for IsEqual {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("equal? function expects exactly two arguments");
        }

        let arg0 = borrow_value(&args[0]);
        let arg0 = downcast_compareable_eq(&arg0).ok_or(InterpreterError::new(
            "equal? function expects a comparable value as the first argument",
        ))?;

        let is_equal = arg0.is_equal(&args[1]);
        Ok(new_valueref(BoolValue { value: is_equal }))
    }
}

fn downcast_compareable_eq<'a>(value: &'a Ref<dyn Value>) -> Option<&'a dyn ComparableEq> {
    match value.get_type() {
        ValueType::Str => Some(value.as_any().downcast_ref::<StrValue>().unwrap()),
        ValueType::Int => Some(value.as_any().downcast_ref::<IntValue>().unwrap()),
        ValueType::Bool => Some(value.as_any().downcast_ref::<BoolValue>().unwrap()),
        _ => None,
    }
}

pub struct Write {}

impl Write {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Write {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Write { line_break: false });
        }
        Ok(new_valueref(NilValue {}))
    }
}

pub struct WriteLn {}

impl WriteLn {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for WriteLn {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Write { line_break: true });
        }
        Ok(new_valueref(NilValue {}))
    }
}

pub struct Display_ {}

impl Display_ {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Display_ {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Display { line_break: false });
        }
        Ok(new_valueref(NilValue {}))
    }
}

pub struct DisplayLn {}

impl DisplayLn {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for DisplayLn {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Display { line_break: true });
        }
        Ok(new_valueref(NilValue {}))
    }
}

enum PrintMode {
    Write { line_break: bool },
    Display { line_break: bool },
}

fn print_value(value: &ValueRef, mode: PrintMode) {
    match mode {
        PrintMode::Write { line_break } => {
            print!("{}", value.borrow());
            if line_break {
                println!();
            }
        }
        PrintMode::Display { line_break } => {
            let value_str = format!("{}", value.borrow());
            let value_str = value_str.trim_matches('"');
            print!("{}", value_str);
            if line_break {
                println!();
            }
        }
    }
}

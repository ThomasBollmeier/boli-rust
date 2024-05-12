use std::{
    cell::{Ref, RefCell},
    fmt::Debug,
    io::{stdout, Write},
    rc::Rc,
};

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

pub trait Input: Debug {
    fn read_line(&mut self) -> String;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub type InputRef = Rc<RefCell<dyn Input>>;

#[derive(Debug)]
pub struct StdInput {}

impl StdInput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Input for StdInput {
    fn read_line(&mut self) -> String {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        input.trim_end().to_string()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct ReadLine {
    input: InputRef,
}

impl ReadLine {
    pub fn new(input: &InputRef) -> Self {
        Self {
            input: input.clone(),
        }
    }
}

impl Callable for ReadLine {
    fn call(&self, _args: &Vec<ValueRef>) -> EvalResult {
        Ok(new_valueref(StrValue {
            value: self.input.borrow_mut().read_line(),
        }))
    }
}

pub trait Output: Debug {
    fn print(&mut self, text: &str);
    fn print_line(&mut self, text: &str) {
        self.print(text);
        self.print("\n");
    }
    fn as_any(&self) -> &dyn std::any::Any;
}

pub type OutputRef = Rc<RefCell<dyn Output>>;

#[derive(Debug)]
pub struct StdOutput {}

impl StdOutput {
    pub fn new() -> Self {
        Self {}
    }
}

impl Output for StdOutput {
    fn print(&mut self, text: &str) {
        print!("{}", text);
        stdout().flush().unwrap();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Write_ {
    output: OutputRef,
}

impl Write_ {
    pub fn new(output: &OutputRef) -> Self {
        Self {
            output: output.clone(),
        }
    }
}

impl Callable for Write_ {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Write { line_break: false }, &self.output);
        }
        Ok(new_valueref(NilValue {}))
    }
}

pub struct WriteLn {
    output: OutputRef,
}

impl WriteLn {
    pub fn new(output: &OutputRef) -> Self {
        Self {
            output: output.clone(),
        }
    }
}

impl Callable for WriteLn {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Write { line_break: true }, &self.output);
        }
        Ok(new_valueref(NilValue {}))
    }
}

pub struct Display_ {
    output: OutputRef,
}

impl Display_ {
    pub fn new(output: &OutputRef) -> Self {
        Self {
            output: output.clone(),
        }
    }
}

impl Callable for Display_ {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Display { line_break: false }, &self.output);
        }
        Ok(new_valueref(NilValue {}))
    }
}

pub struct DisplayLn {
    output: OutputRef,
}

impl DisplayLn {
    pub fn new(output: &OutputRef) -> Self {
        Self {
            output: output.clone(),
        }
    }
}

impl Callable for DisplayLn {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        for arg in args {
            print_value(arg, PrintMode::Display { line_break: true }, &self.output);
        }
        Ok(new_valueref(NilValue {}))
    }
}

enum PrintMode {
    Write { line_break: bool },
    Display { line_break: bool },
}

fn print_value(value: &ValueRef, mode: PrintMode, output: &OutputRef) {
    match mode {
        PrintMode::Write { line_break } => {
            output.borrow_mut().print(&format!("{}", value.borrow()));
            if line_break {
                output.borrow_mut().print_line("");
            }
        }
        PrintMode::Display { line_break } => {
            let value_str = format!("{}", value.borrow());
            let value_str = value_str.trim_matches('"');
            output.borrow_mut().print(value_str);
            if line_break {
                output.borrow_mut().print_line("");
            }
        }
    }
}

pub struct Not {}

impl Not {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Not {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("not function expects exactly one argument");
        }

        Ok(new_valueref(BoolValue {
            value: !is_truthy(&args[0]),
        }))
    }
}

pub struct IsNil {}

impl IsNil {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for IsNil {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("nil? function expects exactly one argument");
        }

        Ok(new_valueref(BoolValue {
            value: args[0].borrow().get_type() == ValueType::Nil,
        }))
    }
}

pub struct ErrorFn {}

impl ErrorFn {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for ErrorFn {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("error function expects exactly one argument");
        }

        let error_message = &borrow_value(&args[0]);
        let error_message = downcast_value::<StrValue>(error_message).unwrap();
        Err(InterpreterError::new(&error_message.value))
    }
}

pub fn is_truthy(value: &ValueRef) -> bool {
    let value = &borrow_value(value);
    match value.get_type() {
        ValueType::Nil => false,
        ValueType::Bool => {
            let bool_value = downcast_value::<BoolValue>(value).unwrap();
            bool_value.value
        }
        ValueType::Int => {
            let int_value = downcast_value::<IntValue>(value).unwrap();
            int_value.value != 0
        }
        ValueType::Vector => {
            let list_value = downcast_value::<VectorValue>(value).unwrap();
            !list_value.elements.is_empty()
        }
        _ => true,
    }
}

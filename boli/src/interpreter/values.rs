use std::error::Error;
use std::fmt::{Debug, Display};
use std::rc::Rc;

#[derive(PartialEq, Debug)]
pub enum ValueType {
    Nil,
    Int,
    Real,
    BuiltInFunction,
}

pub trait Value: Display + Debug {
    fn get_type(&self) -> ValueType;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub fn downcast_value<T: 'static>(value: &Rc<dyn Value>) -> Option<&T> {
    value.as_any().downcast_ref::<T>()
}

#[derive(Debug)]
pub struct NilValue {}

impl Value for NilValue {
    fn get_type(&self) -> ValueType {
        ValueType::Nil
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for NilValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nil")
    }
}

#[derive(Debug)]
pub struct IntValue {
    pub value: i64,
}

impl Value for IntValue {
    fn get_type(&self) -> ValueType {
        ValueType::Int
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug)]
pub struct RealValue {
    pub value: f64,
}

impl Value for RealValue {
    fn get_type(&self) -> ValueType {
        ValueType::Real
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for RealValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value_str = format!("{:?}", self.value).replace(".", ",");
        write!(f, "{value_str}")
    }
}

pub type EvalResult = Result<Rc<dyn Value>, InterpreterError>;

pub trait Callable {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult;
}

pub struct BuiltInFunctionValue {
    pub name: String,
    pub function: Rc<dyn Callable>,
}

impl BuiltInFunctionValue {
    pub fn new(name: &str, function: &Rc<dyn Callable>) -> Self {
        Self {
            name: name.to_string(),
            function: function.clone(),
        }
    }
}

impl Display for BuiltInFunctionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl Debug for BuiltInFunctionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl Value for BuiltInFunctionValue {
    fn get_type(&self) -> ValueType {
        ValueType::BuiltInFunction
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Callable for BuiltInFunctionValue {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        self.function.call(args)
    }
}

#[derive(Debug)]
pub struct InterpreterError {
    pub message: String,
}

impl InterpreterError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InterpreterError {
    fn description(&self) -> &str {
        &self.message
    }
}

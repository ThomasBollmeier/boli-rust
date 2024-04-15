use std::fmt::{Debug, Display};
use std::rc::Rc;

pub enum ValueType {
    Nil,
    Int,
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

pub trait Callable: Debug {
    fn call(&self, args: Vec<Rc<dyn Value>>) -> Rc<dyn Value>;
}

#[derive(Debug)]
pub struct BuiltInFunction {
    pub name: String,
    pub function: Rc<dyn Callable>,
}

impl Value for BuiltInFunction {
    fn get_type(&self) -> ValueType {
        ValueType::BuiltInFunction
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Display for BuiltInFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

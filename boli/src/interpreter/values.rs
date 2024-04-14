use std::fmt::{Debug, Display};

pub enum ValueType {
    Nil,
    Int,
}

pub trait Value: Display + Debug {
    fn get_type(&self) -> ValueType;
}

#[derive(Debug)]
pub struct NilValue {}

impl Value for NilValue {
    fn get_type(&self) -> ValueType {
        ValueType::Nil
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
}

impl Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

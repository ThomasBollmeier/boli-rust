use super::{downcast_value, Callable, InterpreterError, PairValue, ValueType};

pub struct IsPair {}

impl IsPair {
    pub fn new() -> Self {
        IsPair {}
    }
}

impl Callable for IsPair {
    fn call(&self, args: &Vec<super::ValueRef>) -> super::EvalResult {
        if args.len() != 1 {
            return Err(InterpreterError::new(
                "pair? function expects exactly one argument",
            ));
        }

        Ok(super::new_valueref(super::BoolValue {
            value: args[0].borrow().get_type() == super::ValueType::Pair,
        }))
    }
}

pub struct Car {}

impl Car {
    pub fn new() -> Self {
        Car {}
    }
}

impl Callable for Car {
    fn call(&self, args: &Vec<super::ValueRef>) -> super::EvalResult {
        if args.len() != 1 {
            return Err(InterpreterError::new("car requires exactly one argument"));
        }

        if args[0].borrow().get_type() != ValueType::Pair {
            return Err(InterpreterError::new("car requires a pair as argument"));
        }

        let arg0 = args[0].borrow();
        let pair = downcast_value::<PairValue>(&arg0).unwrap();

        Ok(pair.left.clone())
    }
}

pub struct Cdr {}

impl Cdr {
    pub fn new() -> Self {
        Cdr {}
    }
}

impl Callable for Cdr {
    fn call(&self, args: &Vec<super::ValueRef>) -> super::EvalResult {
        if args.len() != 1 {
            return Err(InterpreterError::new("cdr requires exactly one argument"));
        }

        if args[0].borrow().get_type() != ValueType::Pair {
            return Err(InterpreterError::new("cdr requires a pair as argument"));
        }

        let arg0 = args[0].borrow();
        let pair = downcast_value::<PairValue>(&arg0).unwrap();

        Ok(pair.right.clone())
    }
}

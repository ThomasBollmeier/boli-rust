use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::environment::{Environment, EnvironmentBuilder};
use crate::interpreter::module_mgmt::extension::{new_extension, new_extension_dir, ExtensionRef};
use crate::interpreter::prelude::load_module_code;
use crate::interpreter::{
    downcast_value, new_valueref, BoolValue, EvalResult, InterpreterError, PairValue, ValueRef,
    ValueType,
};
use crate::interpreter::{error, Callable};

pub fn create_list_extension(vector: &ExtensionRef) -> ExtensionRef {
    let core_env = EnvironmentBuilder::new().build();
    let env = EnvironmentBuilder::new().parent(&core_env).build();

    env.borrow_mut()
        .set_callable("pair-cons", &Rc::new(PairCons::new()));
    env.borrow_mut()
        .set_callable("pair?", &Rc::new(IsPair::new()));
    env.borrow_mut().set_callable("car", &Rc::new(Car::new()));
    env.borrow_mut().set_callable("cdr", &Rc::new(Cdr::new()));

    let deps = new_extension_dir("deps");
    deps.borrow_mut().add_extension(vector);

    let mut search_dirs = env.borrow().get_module_search_dirs();
    search_dirs.push(deps);

    Environment::set_module_search_dirs(&env, &search_dirs);

    let values = load_module_code(&env, include_str!("list.boli")).unwrap_or(HashMap::new());

    new_extension("list", values)
}

pub struct PairCons {}

impl PairCons {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for PairCons {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("pair-cons function expects exactly two arguments");
        }

        Ok(new_valueref(PairValue {
            left: args[0].clone(),
            right: args[1].clone(),
        }))
    }
}

struct IsPair {}

impl IsPair {
    fn new() -> Self {
        IsPair {}
    }
}

impl Callable for IsPair {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return Err(InterpreterError::new(
                "pair? function expects exactly one argument",
            ));
        }

        Ok(new_valueref(BoolValue {
            value: args[0].borrow().get_type() == ValueType::Pair,
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
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

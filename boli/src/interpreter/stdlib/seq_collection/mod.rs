use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::environment::Environment;
use crate::interpreter::module_mgmt::extension::{new_extension, new_extension_dir, ExtensionRef};
use crate::interpreter::sequence::StreamValue;
use crate::interpreter::stdlib::load_module_code;
use crate::interpreter::{
    borrow_value, downcast_value, error, new_valueref, Callable, EvalResult, IntValue, PairValue,
    ValueRef, ValueType,
};

use super::list::{Car, Cdr};
use super::string::{StrConcat, StrSub};
use super::vector::{VecConcat, VecCons, VecHead, VecTail};

pub fn create_seq_collection_extension(extension_deps: &Vec<ExtensionRef>) -> ExtensionRef {
    let core_env = Environment::new_ref();
    let mut env = Environment::with_parent(&core_env);

    env.set_callable("head", &Rc::new(Head::new()));
    env.set_callable("tail", &Rc::new(Tail::new()));
    env.set_callable("cons", &Rc::new(Cons::new()));
    env.set_callable("concat", &Rc::new(Concat::new()));

    let collection_env = Rc::new(RefCell::new(env));

    let deps = new_extension_dir("deps");
    for dep in extension_deps {
        deps.borrow_mut().add_extension(dep);
    }

    let mut search_dirs = collection_env.borrow().get_module_search_dirs();
    search_dirs.push(deps);
    Environment::set_module_search_dirs(&collection_env, &search_dirs);

    let values =
        load_module_code(&collection_env, include_str!("seqcol.boli")).unwrap_or(HashMap::new());

    new_extension("seqcol", values)
}

struct Head {}

impl Head {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Head {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("head function expects exactly one argument");
        }

        let value_type = args[0].borrow().get_type();

        match value_type {
            ValueType::Vector => VecHead::new().call(args),
            ValueType::Pair => Car::new().call(args),
            ValueType::Stream => {
                let sequence = args[0].clone();
                let sequence = borrow_value(&sequence);
                let mut sequence = downcast_value::<StreamValue>(&sequence).unwrap().clone();
                match sequence.next() {
                    Some(head) => Ok(head),
                    None => error("head function expects a non-empty stream "),
                }
            }
            ValueType::Str => {
                let start = new_valueref(IntValue { value: 0 });
                let length = new_valueref(IntValue { value: 1 });
                let args = vec![args[0].clone(), start, length];
                StrSub::new().call(&args)
            }
            _ => error("head function expects a sequential collection"),
        }
    }
}

struct Tail {}

impl Tail {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Tail {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("tail function expects exactly one argument");
        }

        let value_type = args[0].borrow().get_type();

        match value_type {
            ValueType::Vector => VecTail::new().call(args),
            ValueType::Pair => Cdr::new().call(args),
            ValueType::Stream => {
                let sequence = args[0].clone();
                let sequence = borrow_value(&sequence);
                let mut sequence = downcast_value::<StreamValue>(&sequence).unwrap().clone();
                sequence.next();
                Ok(new_valueref(sequence))
            }
            ValueType::Str => {
                let start = new_valueref(IntValue { value: 1 });
                let args = vec![args[0].clone(), start];
                StrSub::new().call(&args)
            }
            _ => error("tail function expects a sequential collection"),
        }
    }
}

struct Cons {}

impl Cons {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Cons {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("cons function expects exactly two arguments");
        }

        let second_arg = args[1].borrow();

        match second_arg.get_type() {
            ValueType::Vector => VecCons::new().call(args),
            _ => Ok(new_valueref(PairValue {
                left: args[0].clone(),
                right: args[1].clone(),
            })),
        }
    }
}

pub struct Concat {}

impl Concat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Concat {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let mut unique_type: Option<ValueType> = None;

        for arg in args {
            let arg_type = arg.borrow().get_type();
            if let Some(prev_type) = unique_type {
                if arg_type != prev_type {
                    return error("concat function expects all arguments to be of the same type");
                }
            } else {
                unique_type = Some(arg_type);
            }
        }

        if unique_type.is_none() {
            return error("concat function expects at least one argument");
        }
        let unique_type = unique_type.unwrap();

        match unique_type {
            ValueType::Vector => VecConcat::new().call(args),
            ValueType::Pair => {
                todo!()
            }
            ValueType::Str => StrConcat::new().call(args),
            _ => error("concat function expects arguments to be vectors, lists, or strings"),
        }
    }
}

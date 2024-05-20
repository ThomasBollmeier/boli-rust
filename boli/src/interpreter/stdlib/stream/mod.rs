use std::{collections::HashMap, rc::Rc};

use crate::interpreter::{
    environment::Environment,
    error,
    module_mgmt::extension::{new_extension, ExtensionRef},
    new_valueref,
    stream::StreamValue,
    BoolValue, Callable, EvalResult, ValueRef, ValueType,
};

use super::load_module_code;

pub fn create_stream_extension() -> ExtensionRef {
    let core_env = Environment::new_ref();
    let env = Environment::with_parent(&core_env);

    env.borrow_mut()
        .set_callable("stream?", &Rc::new(IsStream::new()));
    env.borrow_mut()
        .set_callable("vector->stream", &Rc::new(VectorToStream::new()));
    env.borrow_mut()
        .set_callable("iterator", &Rc::new(Iterator::new()));
    env.borrow_mut()
        .set_callable("stream-map", &Rc::new(StreamMap::new()));

    let values = load_module_code(&env, include_str!("stream.boli")).unwrap_or(HashMap::new());

    new_extension("stream", values)
}

struct IsStream {}

impl IsStream {
    fn new() -> Self {
        Self {}
    }
}

impl Callable for IsStream {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("stream? function expects one argument");
        }

        let is_stream = args[0].borrow().get_type() == ValueType::Stream;

        Ok(new_valueref(BoolValue { value: is_stream }))
    }
}

struct VectorToStream {}

impl VectorToStream {
    fn new() -> Self {
        Self {}
    }
}

impl Callable for VectorToStream {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("vector->stream function expects one argument");
        }

        if args[0].borrow().get_type() != ValueType::Vector {
            return error("sequence function expects a list as the argument");
        }

        Ok(new_valueref(StreamValue::new_list(args[0].clone())?))
    }
}

pub struct Iterator {}

impl Iterator {
    fn new() -> Self {
        Self {}
    }
}

impl Callable for Iterator {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("iterator function expects two arguments");
        }

        let (start, next_function) = (&args[0], &args[1]);
        let iterator = StreamValue::new_iterator(next_function.clone(), start.clone())?;

        Ok(new_valueref(iterator))
    }
}

struct StreamMap {}

impl StreamMap {
    fn new() -> Self {
        Self {}
    }
}

impl Callable for StreamMap {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() < 2 {
            return error("map function expects at least two arguments");
        }

        let map_function = args[0].clone();
        let streams = &args[1..].to_vec();
        let mapped = StreamValue::new_mapped(map_function, streams.clone())?;

        Ok(new_valueref(mapped))
    }
}

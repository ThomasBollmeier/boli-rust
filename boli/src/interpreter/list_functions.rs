use super::values::*;
use std::rc::Rc;

fn error(message: &str) -> EvalResult {
    Err(InterpreterError::new(message))
}

pub struct List {}

impl List {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for List {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        Ok(Rc::new(ListValue {
            elements: args.clone(),
        }) as Rc<dyn Value>)
    }
}

pub struct Head {}

impl Head {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Head {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        if args.len() != 1 {
            return error("head function expects exactly one argument");
        }

        match args[0].as_any().downcast_ref::<ListValue>() {
            Some(list) => {
                if list.elements.is_empty() {
                    return error("head function expects a non-empty list");
                }

                Ok(list.elements[0].clone())
            }
            None => error("head function expects a list"),
        }
    }
}

pub struct Tail {}

impl Tail {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Tail {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        if args.len() != 1 {
            return error("tail function expects exactly one argument");
        }

        match args[0].as_any().downcast_ref::<ListValue>() {
            Some(list) => {
                if list.elements.is_empty() {
                    return error("tail function expects a non-empty list");
                }

                Ok(Rc::new(ListValue {
                    elements: list.elements[1..].to_vec(),
                }) as Rc<dyn Value>)
            }
            None => error("tail function expects a list"),
        }
    }
}

pub struct Cons {}

impl Cons {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Cons {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        if args.len() != 2 {
            return error("cons function expects exactly two arguments");
        }

        match args[1].as_any().downcast_ref::<ListValue>() {
            Some(list) => {
                let mut elements = vec![args[0].clone()];
                elements.extend(list.elements.clone());

                Ok(Rc::new(ListValue { elements }) as Rc<dyn Value>)
            }
            None => error("cons function expects a list as the second argument"),
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
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        let mut elements = Vec::new();

        for arg in args {
            match arg.as_any().downcast_ref::<ListValue>() {
                Some(list) => elements.extend(list.elements.clone()),
                None => return error("concat function expects a list as arguments"),
            }
        }

        Ok(Rc::new(ListValue { elements }) as Rc<dyn Value>)
    }
}

pub struct Filter {}

impl Filter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Filter {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        if args.len() != 2 {
            return error("filter function expects exactly two arguments");
        }

        let predicate: &dyn Callable = match args[0].get_type() {
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(&args[0]).unwrap(),
            ValueType::Lambda => downcast_value::<LambdaValue>(&args[0]).unwrap(),
            _ => {
                return error("filter function expects a predicate function as the first argument")
            }
        };

        match args[1].get_type() {
            ValueType::List => {
                let list = downcast_value::<ListValue>(&args[1]).unwrap();
                let mut elements = Vec::new();
                for elem in &list.elements {
                    let result = predicate.call(&vec![elem.clone()])?;
                    match result.get_type() {
                        ValueType::Bool => {
                            if downcast_value::<BoolValue>(&result).unwrap().value {
                                elements.push(elem.clone());
                            }
                        }
                        _ => {
                            return error(
                                "filter function expects a predicate that returns a boolean value",
                            )
                        }
                    }
                }
                Ok(Rc::new(ListValue { elements }) as Rc<dyn Value>)
            }
            _ => return error("filter function expects a list as the second argument"),
        }
    }
}

pub struct Map {}

impl Map {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Map {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        if args.len() < 2 {
            return error("map function expects at least two arguments");
        }

        let function: &dyn Callable = match args[0].get_type() {
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(&args[0]).unwrap(),
            ValueType::Lambda => downcast_value::<LambdaValue>(&args[0]).unwrap(),
            _ => return error("map function expects a function as the first argument"),
        };

        let mut min_size_opt: Option<usize> = None;
        let mut lists = Vec::new();

        for arg in args.iter().skip(1) {
            match arg.get_type() {
                ValueType::List => {
                    let list = downcast_value::<ListValue>(&arg).unwrap();
                    lists.push(list);
                    if let Some(min_size) = min_size_opt {
                        if list.elements.len() < min_size {
                            min_size_opt = Some(list.elements.len());
                        }
                    } else {
                        min_size_opt = Some(list.elements.len());
                    }
                }
                _ => return error("map function expects a list as arguments"),
            }
        }

        let min_size = min_size_opt.unwrap_or(0);

        let mut elements = Vec::new();

        for i in 0..min_size {
            let mut args = Vec::new();
            for list in lists.iter() {
                args.push(list.elements[i].clone());
            }

            let result = function.call(&args)?;
            elements.push(result);
        }

        Ok(Rc::new(ListValue { elements }) as Rc<dyn Value>)
    }
}

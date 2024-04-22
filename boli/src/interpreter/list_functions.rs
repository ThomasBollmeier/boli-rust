use super::values::*;

pub struct List {}

impl List {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for List {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        Ok(new_valueref(ListValue {
            elements: args.clone(),
        }))
    }
}

pub struct Head {}

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

        let first_arg = args[0].borrow();
        match first_arg.as_any().downcast_ref::<ListValue>() {
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("tail function expects exactly one argument");
        }

        let first_arg = args[0].borrow();
        match first_arg.as_any().downcast_ref::<ListValue>() {
            Some(list) => {
                if list.elements.is_empty() {
                    return error("tail function expects a non-empty list");
                }

                Ok(new_valueref(ListValue {
                    elements: list.elements[1..].to_vec(),
                }) as ValueRef)
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("cons function expects exactly two arguments");
        }

        let second_arg = args[1].borrow();
        match second_arg.as_any().downcast_ref::<ListValue>() {
            Some(list) => {
                let mut elements = vec![args[0].clone()];
                elements.extend(list.elements.clone());

                Ok(new_valueref(ListValue { elements }) as ValueRef)
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let mut elements = Vec::new();

        for arg in args {
            let arg = arg.borrow();
            match arg.as_any().downcast_ref::<ListValue>() {
                Some(list) => elements.extend(list.elements.clone()),
                None => return error("concat function expects a list as arguments"),
            }
        }

        Ok(new_valueref(ListValue { elements }) as ValueRef)
    }
}

pub struct Filter {}

impl Filter {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Filter {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("filter function expects exactly two arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let predicate: &dyn Callable = match arg0.get_type() {
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(arg0).unwrap(),
            ValueType::Lambda => downcast_value::<LambdaValue>(arg0).unwrap(),
            _ => {
                return error("filter function expects a predicate function as the first argument")
            }
        };

        let arg1 = &borrow_value(&args[1]);
        match arg1.get_type() {
            ValueType::List => {
                let list = downcast_value::<ListValue>(arg1).unwrap();
                let mut elements = Vec::new();
                for elem in &list.elements {
                    let result = predicate.call(&vec![elem.clone()])?;
                    let result = borrow_value(&result);
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
                Ok(new_valueref(ListValue { elements }) as ValueRef)
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() < 2 {
            return error("map function expects at least two arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let function: &dyn Callable = match arg0.get_type() {
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(arg0).unwrap(),
            ValueType::Lambda => downcast_value::<LambdaValue>(arg0).unwrap(),
            _ => return error("map function expects a function as the first argument"),
        };

        let mut min_size_opt: Option<usize> = None;

        for arg in args.iter().skip(1) {
            let arg = &borrow_value(&arg);
            match arg.get_type() {
                ValueType::List => {
                    let list = downcast_value::<ListValue>(arg).unwrap();
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
            let mut call_args = Vec::new();
            for arg in args.iter().skip(1) {
                let arg = &borrow_value(&arg);
                let list = downcast_value::<ListValue>(arg).unwrap();
                call_args.push(list.elements[i].clone());
            }

            let result = function.call(&call_args)?;
            elements.push(result);
        }

        Ok(new_valueref(ListValue { elements }) as ValueRef)
    }
}

pub struct ListRef {}

impl ListRef {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for ListRef {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("list-ref function expects exactly two arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let list = match arg0.as_any().downcast_ref::<ListValue>() {
            Some(list) => list,
            None => return error("list-ref function expects a list"),
        };

        let arg1 = &borrow_value(&args[1]);
        let index = match arg1.as_any().downcast_ref::<IntValue>() {
            Some(index) => index.value,
            None => return error("list-ref function expects an integer as the second argument"),
        };

        Ok(list.elements[index as usize].clone())
    }
}

pub struct ListSetBang {}

impl ListSetBang {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for ListSetBang {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 3 {
            return error("list-set! function expects exactly three arguments");
        }

        let arg0 = &mut borrow_mut_value(&args[0]);
        let list = match arg0.as_any_mut().downcast_mut::<ListValue>() {
            Some(list) => list,
            None => return error("list-set! function expects a list"),
        };

        let arg1 = &borrow_value(&args[1]);
        let index = match arg1.as_any().downcast_ref::<IntValue>() {
            Some(index) => index.value,
            None => return error("list-set! function expects an integer as the second argument"),
        };

        let arg2 = args[2].clone();
        list.elements[index as usize] = arg2;

        Ok(new_valueref(ListValue {
            elements: list.elements.clone(),
        }))
    }
}

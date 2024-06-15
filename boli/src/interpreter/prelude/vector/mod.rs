use std::rc::Rc;

use crate::interpreter::{
    borrow_mut_value, borrow_value, downcast_value,
    environment::EnvironmentBuilder,
    error,
    module_mgmt::extension::{new_extension, ExtensionRef},
    new_valueref, BoolValue, Callable, EvalResult, IntValue, ValueRef, ValueType, VectorValue,
};

pub fn create_vector_extension() -> ExtensionRef {
    let core_env = EnvironmentBuilder::new().build();
    let env = EnvironmentBuilder::new().parent(&core_env).build();
    env.borrow_mut()
        .set_callable("vector", &Rc::new(Vector::new()));
    env.borrow_mut()
        .set_callable("vector?", &Rc::new(IsVector::new()));
    env.borrow_mut()
        .set_callable("vector-count", &Rc::new(VecCount::new()));
    env.borrow_mut()
        .set_callable("vector-head", &Rc::new(VecHead::new()));
    env.borrow_mut()
        .set_callable("vector-tail", &Rc::new(VecTail::new()));
    env.borrow_mut()
        .set_callable("vector-cons", &Rc::new(VecCons::new()));
    env.borrow_mut()
        .set_callable("vector-concat", &Rc::new(VecConcat::new()));
    env.borrow_mut()
        .set_callable("vector-ref", &Rc::new(VecRef::new()));
    env.borrow_mut()
        .set_callable("vector-set!", &Rc::new(VecSetBang::new()));
    env.borrow_mut()
        .set_callable("vector-remove!", &Rc::new(VecRemoveBang::new()));

    let exported_values = env.borrow().get_exported_values();

    new_extension("vector", exported_values)
}

struct Vector {}

impl Vector {
    fn new() -> Self {
        Self {}
    }
}

impl Callable for Vector {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        Ok(new_valueref(VectorValue {
            elements: args.clone(),
        }))
    }
}

struct IsVector {}

impl IsVector {
    fn new() -> Self {
        Self {}
    }
}

impl Callable for IsVector {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("vector? function expects exactly one argument");
        }

        let arg0 = borrow_value(&args[0]);
        Ok(new_valueref(BoolValue {
            value: arg0.get_type() == ValueType::Vector,
        }))
    }
}

pub struct VecCount {}

impl VecCount {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecCount {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("vector-count function expects exactly one argument");
        }

        let value_type = args[0].borrow().get_type();

        match value_type {
            ValueType::Vector => {
                let list = &borrow_value(&args[0]);
                let list = downcast_value::<VectorValue>(list).unwrap();
                Ok(new_valueref(IntValue {
                    value: list.elements.len() as i64,
                }))
            }
            _ => error("vector-count function expects a vector"),
        }
    }
}

pub struct VecHead {}

impl VecHead {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecHead {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("head function expects exactly one argument");
        }

        let value_type = args[0].borrow().get_type();

        match value_type {
            ValueType::Vector => {
                let list = &borrow_value(&args[0]);
                let list = downcast_value::<VectorValue>(list).unwrap();
                if list.elements.is_empty() {
                    return error("head function expects a non-empty list");
                }
                Ok(list.elements[0].clone())
            }
            _ => error("head function expects a list"),
        }
    }
}

pub struct VecTail {}

impl VecTail {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecTail {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("vector-tail function expects exactly one argument");
        }

        let value_type = args[0].borrow().get_type();

        match value_type {
            ValueType::Vector => {
                let list = &borrow_value(&args[0]);
                let list = downcast_value::<VectorValue>(list).unwrap();
                if list.elements.is_empty() {
                    return error("vector-tail function expects a non-empty vector");
                }
                Ok(new_valueref(VectorValue {
                    elements: list.elements[1..].to_vec(),
                }))
            }
            _ => error("vector-tail function expects a vector"),
        }
    }
}

pub struct VecCons {}

impl VecCons {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecCons {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("vector-cons function expects exactly two arguments");
        }

        let second_arg = args[1].borrow();
        match second_arg.as_any().downcast_ref::<VectorValue>() {
            Some(list) => {
                let mut elements = vec![args[0].clone()];
                elements.extend(list.elements.clone());

                Ok(new_valueref(VectorValue { elements }) as ValueRef)
            }
            None => error("vector-cons function expects a vector as second argument"),
        }
    }
}

pub struct VecConcat {}

impl VecConcat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecConcat {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let mut elements = Vec::new();

        for arg in args {
            let arg = arg.borrow();
            match arg.as_any().downcast_ref::<VectorValue>() {
                Some(list) => elements.extend(list.elements.clone()),
                None => return error("vector-concat function expects vectors as arguments"),
            }
        }

        Ok(new_valueref(VectorValue { elements }) as ValueRef)
    }
}

struct VecRef {}

impl VecRef {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecRef {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("vector-ref function expects exactly two arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let list = match arg0.as_any().downcast_ref::<VectorValue>() {
            Some(list) => list,
            None => return error("vector-ref function expects a list"),
        };

        let arg1 = &borrow_value(&args[1]);
        let index = match arg1.as_any().downcast_ref::<IntValue>() {
            Some(index) => index.value,
            None => return error("vector-ref function expects an integer as the second argument"),
        };

        Ok(list.elements[index as usize].clone())
    }
}

struct VecSetBang {}

impl VecSetBang {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecSetBang {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 3 {
            return error("vector-set! function expects exactly three arguments");
        }

        let arg0 = &mut borrow_mut_value(&args[0]);
        let list = match arg0.as_any_mut().downcast_mut::<VectorValue>() {
            Some(list) => list,
            None => return error("vector-set! function expects a list"),
        };

        let arg1 = &borrow_value(&args[1]);
        let index = match arg1.as_any().downcast_ref::<IntValue>() {
            Some(index) => index.value,
            None => return error("vector-set! function expects an integer as the second argument"),
        };

        let arg2 = args[2].clone();
        list.elements[index as usize] = arg2;

        Ok(new_valueref(VectorValue {
            elements: list.elements.clone(),
        }))
    }
}

struct VecRemoveBang {}

impl VecRemoveBang {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for VecRemoveBang {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("vector-remove! function expects exactly two arguments");
        }

        let arg0 = &mut borrow_mut_value(&args[0]);
        let vector = match arg0.as_any_mut().downcast_mut::<VectorValue>() {
            Some(vector) => vector,
            None => return error("vector-remove! function expects a list"),
        };

        let arg1 = &borrow_value(&args[1]);
        let index = match arg1.as_any().downcast_ref::<IntValue>() {
            Some(index) => index.value,
            None => {
                return error("vector-remove! function expects an integer as the second argument")
            }
        };

        vector.elements.remove(index as usize);

        Ok(new_valueref(VectorValue {
            elements: vector.elements.clone(),
        }))
    }
}

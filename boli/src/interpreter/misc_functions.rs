use std::cell::Ref;

use super::values::*;

pub struct IsEqual {}

impl IsEqual {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for IsEqual {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("equal? function expects exactly two arguments");
        }

        let arg0 = borrow_value(&args[0]);
        let arg0 = downcast_compareable_eq(&arg0).ok_or(InterpreterError::new(
            "equal? function expects a comparable value as the first argument",
        ))?;

        let is_equal = arg0.is_equal(&args[1]);
        Ok(new_valueref(BoolValue { value: is_equal }))
    }
}

fn downcast_compareable_eq<'a>(value: &'a Ref<dyn Value>) -> Option<&'a dyn ComparableEq> {
    match value.get_type() {
        ValueType::Str => Some(value.as_any().downcast_ref::<StrValue>().unwrap()),
        ValueType::Int => Some(value.as_any().downcast_ref::<IntValue>().unwrap()),
        ValueType::Bool => Some(value.as_any().downcast_ref::<BoolValue>().unwrap()),
        _ => None,
    }
}

use std::cell::Ref;

use super::values::*;

pub struct Count {}

impl Count {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Count {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("count function expects exactly one argument");
        }

        let arg0 = borrow_value(&args[0]);

        if let Some(countable) = downcast_countable(&arg0) {
            Ok(new_valueref(IntValue {
                value: countable.count() as i64,
            }))
        } else {
            error("count function expects a string or a list")
        }
    }
}

pub struct IsEmpty {}

impl IsEmpty {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for IsEmpty {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let count = Count::new();
        let count_result = count.call(args)?;
        let count_result = borrow_value(&count_result);
        let count_result = downcast_value::<IntValue>(&count_result).unwrap();

        Ok(new_valueref(BoolValue {
            value: count_result.value == 0,
        }))
    }
}

fn downcast_countable<'a>(value: &'a Ref<dyn Value>) -> Option<&'a dyn Countable> {
    match value.get_type() {
        ValueType::Str => Some(value.as_any().downcast_ref::<StrValue>().unwrap()),
        ValueType::List => Some(value.as_any().downcast_ref::<ListValue>().unwrap()),
        _ => None,
    }
}

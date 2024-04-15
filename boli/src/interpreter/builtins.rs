use super::values::*;
use std::rc::Rc;

pub struct Add {}

impl Add {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Add {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        let mut result = 0;
        for arg in args {
            if let Some(int_value) = downcast_value::<IntValue>(&arg) {
                result += int_value.value;
            }
        }

        Ok(Rc::new(IntValue { value: result }))
    }
}

fn is_numeric(value: &Rc<dyn Value>) -> bool {
    let value_type = value.get_type();
    value_type == ValueType::Int || value_type == ValueType::Real
}

fn is_int(value: &Rc<dyn Value>) -> bool {
    value.get_type() == ValueType::Int
}

use super::values::*;
use std::rc::Rc;

#[derive(Debug)]
pub struct Add {}

impl Callable for Add {
    fn call(&self, args: Vec<Rc<dyn Value>>) -> Rc<dyn Value> {
        let mut result = 0;
        for arg in args {
            if let Some(int_value) = downcast_value::<IntValue>(&arg) {
                result += int_value.value;
            }
        }

        Rc::new(IntValue { value: result })
    }
}

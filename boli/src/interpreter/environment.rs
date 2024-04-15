use super::values::*;
use super::{builtins::*, BuiltInFunction};
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    pub env: HashMap<String, Rc<dyn Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut result = Self {
            env: HashMap::new(),
            parent: None,
        };
        result.init_builtins();

        result
    }

    pub fn with_parent(parent: &Rc<Environment>) -> Self {
        Self {
            env: HashMap::new(),
            parent: Some(parent.clone()),
        }
    }

    pub fn get(&self, key: &str) -> Option<Rc<dyn Value>> {
        if let Some(value) = self.env.get(key) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.get(key);
        }

        None
    }

    pub fn set(&mut self, key: String, value: Rc<dyn Value>) {
        self.env.insert(key, value);
    }

    fn init_builtins(&mut self) {
        let add: Rc<dyn Callable> = Rc::new(Add {});
        self.set_builtins("+", &add);
    }

    fn set_builtins(&mut self, name: &str, function: &Rc<dyn Callable>) {
        self.set(
            name.to_string(),
            Rc::new(BuiltInFunction {
                name: name.to_string(),
                function: function.clone(),
            }),
        );
    }
}

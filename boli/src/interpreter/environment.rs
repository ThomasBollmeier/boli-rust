use crate::interpreter::values::Value;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    pub env: HashMap<String, Rc<dyn Value>>,
    parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            env: HashMap::new(),
            parent: None,
        }
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
}

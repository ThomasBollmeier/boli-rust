use super::list_functions::*;
use super::number_functions::*;
use super::values::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    pub env: HashMap<String, Rc<dyn Value>>,
    parent: Option<Rc<RefCell<Environment>>>,
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

    pub fn with_parent(parent: &Rc<RefCell<Environment>>) -> Self {
        Self {
            env: HashMap::new(),
            parent: Some(parent.clone()),
        }
    }

    pub fn get_parent(&self) -> Option<Rc<RefCell<Environment>>> {
        self.parent.clone()
    }

    pub fn get(&self, key: &str) -> Option<Rc<dyn Value>> {
        if let Some(value) = self.env.get(key) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(key);
        }

        None
    }

    pub fn set(&mut self, key: String, value: Rc<dyn Value>) {
        self.env.insert(key, value);
    }

    fn init_builtins(&mut self) {
        self.set_builtins("+", &Rc::new(Add::new()));
        self.set_builtins("-", &Rc::new(Sub::new()));
        self.set_builtins("*", &Rc::new(Mul::new()));
        self.set_builtins("/", &Rc::new(Div::new()));
        self.set_builtins("^", &Rc::new(Pow::new()));
        self.set_builtins("%", &Rc::new(Rem::new()));
        self.set_builtins("=", &Rc::new(Eq::new()));
        self.set_builtins(">", &Rc::new(Gt::new()));
        self.set_builtins(">=", &Rc::new(Ge::new()));
        self.set_builtins("<", &Rc::new(Lt::new()));
        self.set_builtins("<=", &Rc::new(Le::new()));

        self.set_builtins("list", &Rc::new(List::new()));
        self.set_builtins("head", &Rc::new(Head::new()));
        self.set_builtins("tail", &Rc::new(Tail::new()));
        self.set_builtins("cons", &Rc::new(Cons::new()));
        self.set_builtins("concat", &Rc::new(Concat::new()));
        self.set_builtins("filter", &Rc::new(Filter::new()));
        self.set_builtins("map", &Rc::new(Map::new()));
    }

    fn set_builtins<T: Callable + 'static>(&mut self, name: &str, function: &Rc<T>) {
        self.set(
            name.to_string(),
            Rc::new(BuiltInFunctionValue {
                name: name.to_string(),
                function: function.clone(),
            }),
        );
    }
}

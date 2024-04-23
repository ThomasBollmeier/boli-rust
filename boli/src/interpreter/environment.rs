use super::count_functions::*;
use super::list_functions::*;
use super::misc_functions::*;
use super::number_functions::*;
use super::string_functions::*;
use super::values::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    pub env: HashMap<String, ValueRef>,
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

    pub fn get(&self, key: &str) -> Option<ValueRef> {
        if let Some(value) = self.env.get(key) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(key);
        }

        None
    }

    pub fn set(&mut self, key: String, value: ValueRef) {
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
        self.set_builtins("list-ref", &Rc::new(ListRef::new()));
        self.set_builtins("list-set!", &Rc::new(ListSetBang::new()));

        self.set_builtins("count", &Rc::new(Count::new()));
        self.set_builtins("empty?", &Rc::new(IsEmpty::new()));

        self.set_builtins("str-sub", &Rc::new(StrSub::new()));
        self.set_builtins("str-replace", &Rc::new(StrReplace::new()));
        self.set_builtins("str-concat", &Rc::new(StrConcat::new()));
        self.set_builtins("str-upper", &Rc::new(StrUpper::new()));
        self.set_builtins("str-lower", &Rc::new(StrLower::new()));

        self.set_builtins("equal?", &Rc::new(IsEqual::new()));
        self.set_builtins("write", &Rc::new(Write::new()));
        self.set_builtins("writeln", &Rc::new(WriteLn::new()));
        self.set_builtins("display", &Rc::new(Display_::new()));
        self.set_builtins("displayln", &Rc::new(DisplayLn::new()));
    }

    fn set_builtins<T: Callable + 'static>(&mut self, name: &str, function: &Rc<T>) {
        self.set(
            name.to_string(),
            new_valueref(BuiltInFunctionValue {
                name: name.to_string(),
                function: function.clone(),
            }),
        );
    }
}

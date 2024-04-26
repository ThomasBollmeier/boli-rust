use super::count_functions::*;
use super::list_functions::*;
use super::misc_functions::*;
use super::number_functions::*;
use super::string_functions::*;
use super::struct_functions::*;
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
        self.set_builtin("+", &Rc::new(Add::new()));
        self.set_builtin("-", &Rc::new(Sub::new()));
        self.set_builtin("*", &Rc::new(Mul::new()));
        self.set_builtin("/", &Rc::new(Div::new()));
        self.set_builtin("^", &Rc::new(Pow::new()));
        self.set_builtin("%", &Rc::new(Rem::new()));
        self.set_builtin("=", &Rc::new(Eq::new()));
        self.set_builtin(">", &Rc::new(Gt::new()));
        self.set_builtin(">=", &Rc::new(Ge::new()));
        self.set_builtin("<", &Rc::new(Lt::new()));
        self.set_builtin("<=", &Rc::new(Le::new()));

        self.set_builtin("list", &Rc::new(List::new()));
        self.set_builtin("head", &Rc::new(Head::new()));
        self.set_builtin("tail", &Rc::new(Tail::new()));
        self.set_builtin("cons", &Rc::new(Cons::new()));
        self.set_builtin("concat", &Rc::new(Concat::new()));
        self.set_builtin("filter", &Rc::new(Filter::new()));
        self.set_builtin("map", &Rc::new(Map::new()));
        self.set_builtin("list-ref", &Rc::new(ListRef::new()));
        self.set_builtin("list-set!", &Rc::new(ListSetBang::new()));

        self.set_builtin("count", &Rc::new(Count::new()));
        self.set_builtin("empty?", &Rc::new(IsEmpty::new()));

        self.set_builtin("str-sub", &Rc::new(StrSub::new()));
        self.set_builtin("str-replace", &Rc::new(StrReplace::new()));
        self.set_builtin("str-concat", &Rc::new(StrConcat::new()));
        self.set_builtin("str-upper", &Rc::new(StrUpper::new()));
        self.set_builtin("str-lower", &Rc::new(StrLower::new()));

        self.set_builtin("equal?", &Rc::new(IsEqual::new()));
        self.set_builtin("write", &Rc::new(Write::new()));
        self.set_builtin("writeln", &Rc::new(WriteLn::new()));
        self.set_builtin("display", &Rc::new(Display_::new()));
        self.set_builtin("displayln", &Rc::new(DisplayLn::new()));

        self.set_builtin("struct-get", &Rc::new(StructGet::new()));
        self.set_builtin("struct-set!", &Rc::new(StructSet::new()));
        self.set_builtin("create-hash-table", &Rc::new(CreateHashTable::new()));
    }

    pub fn set_builtin<T: Callable + 'static>(&mut self, name: &str, function: &Rc<T>) {
        self.set(
            name.to_string(),
            new_valueref(BuiltInFunctionValue {
                name: name.to_string(),
                function: function.clone(),
            }),
        );
    }
}

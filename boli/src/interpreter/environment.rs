use super::count_functions::*;
use super::list_functions::*;
use super::misc_functions::*;
use super::module_mgmt::file_system::new_directory;
use super::module_mgmt::module_loader::RequireFn;
use super::module_mgmt::ModuleDirRef;
use super::number_functions::*;
use super::pair_functions::Car;
use super::pair_functions::Cdr;
use super::string_functions::*;
use super::struct_functions::*;
use super::values::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Environment {
    pub env: HashMap<String, EnvEntry>,
    module_search_dirs: Option<Vec<ModuleDirRef>>,
    input: Option<InputRef>,
    output: Option<OutputRef>,
    parent: Option<EnvironmentRef>,
}

pub struct EnvEntry {
    value: ValueRef,
    owned: bool,
}

pub type EnvironmentRef = Rc<RefCell<Environment>>;

impl Environment {
    pub fn new_ref() -> EnvironmentRef {
        let ret = Rc::new(RefCell::new(Self {
            env: HashMap::new(),
            module_search_dirs: None,
            input: None,
            output: None,
            parent: None,
        }));
        Self::init_builtins(&ret);

        ret
    }

    pub fn ref_with_search_dirs_and_output(
        search_dirs: &Vec<ModuleDirRef>,
        output: &OutputRef,
    ) -> EnvironmentRef {
        let ret = Rc::new(RefCell::new(Self {
            env: HashMap::new(),
            module_search_dirs: Some(search_dirs.clone()),
            input: None,
            output: Some(output.clone()),
            parent: None,
        }));
        Self::init_builtins(&ret);

        ret
    }

    pub fn with_parent(parent: &EnvironmentRef) -> Self {
        Self {
            env: HashMap::new(),
            module_search_dirs: None,
            input: None,
            output: None,
            parent: Some(parent.clone()),
        }
    }

    pub fn set_module_search_dirs(env: &EnvironmentRef, dirs: &Vec<ModuleDirRef>) {
        env.borrow_mut().module_search_dirs = Some(dirs.clone());
        Self::init_require_builtin(env);
    }

    pub fn get_module_search_dirs(&self) -> Vec<ModuleDirRef> {
        match self.module_search_dirs {
            Some(ref dirs) => dirs.clone(),
            None => {
                if let Some(parent) = &self.parent {
                    return parent.borrow().get_module_search_dirs();
                } else {
                    vec![new_directory(".", "")]
                }
            }
        }
    }

    pub fn get_input(&self) -> InputRef {
        match &self.input {
            Some(input) => input.clone(),
            None => {
                if let Some(parent) = &self.parent {
                    return parent.borrow().get_input();
                } else {
                    Rc::new(RefCell::new(StdInput::new()))
                }
            }
        }
    }

    pub fn set_output(env: &EnvironmentRef, output: &OutputRef) {
        env.borrow_mut().output = Some(output.clone());
        Self::init_output_builtins(env);
    }

    pub fn get_output(&self) -> OutputRef {
        match &self.output {
            Some(output) => output.clone(),
            None => {
                if let Some(parent) = &self.parent {
                    return parent.borrow().get_output();
                } else {
                    Rc::new(RefCell::new(StdOutput::new()))
                }
            }
        }
    }

    pub fn get_parent(&self) -> Option<Rc<RefCell<Environment>>> {
        self.parent.clone()
    }

    pub fn get(&self, key: &str) -> Option<ValueRef> {
        if let Some(EnvEntry { value, owned: _ }) = self.env.get(key) {
            return Some(value.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.borrow().get(key);
        }

        None
    }

    pub fn set(&mut self, key: String, value: ValueRef) {
        self.env.insert(key, EnvEntry { value, owned: true }); // true: value is owned by the environment
    }

    fn set_unowned(&mut self, key: String, value: ValueRef) {
        self.env.insert(
            key,
            EnvEntry {
                value,
                owned: false,
            },
        ); // false: value is not owned by the environment
    }

    pub fn get_exported_values(&self) -> HashMap<String, ValueRef> {
        HashMap::from(
            self.env
                .iter()
                .filter(|(_, EnvEntry { value: _, owned })| *owned)
                .map(|(key, EnvEntry { value, owned: _ })| (key.clone(), value.clone()))
                .collect::<HashMap<String, ValueRef>>(),
        )
    }

    pub fn import_values(&mut self, values: HashMap<String, ValueRef>) {
        for (key, value) in values {
            self.env.insert(
                key,
                EnvEntry {
                    value,
                    owned: false,
                },
            ); // false: value is not owned by the environment
        }
    }

    pub fn import_values_with_alias(&mut self, values: HashMap<String, ValueRef>, alias: &str) {
        for (key, value) in values {
            self.env.insert(
                format!("{}::{}", alias, key),
                EnvEntry {
                    value,
                    owned: false,
                },
            ); // false: value is not owned by the environment
        }
    }

    fn init_builtins(env: &EnvironmentRef) {
        env.borrow_mut().set_builtin("+", &Rc::new(Add::new()));
        env.borrow_mut().set_builtin("-", &Rc::new(Sub::new()));
        env.borrow_mut().set_builtin("*", &Rc::new(Mul::new()));
        env.borrow_mut().set_builtin("/", &Rc::new(Div::new()));
        env.borrow_mut().set_builtin("^", &Rc::new(Pow::new()));
        env.borrow_mut().set_builtin("%", &Rc::new(Rem::new()));
        env.borrow_mut().set_builtin("=", &Rc::new(Eq::new()));
        env.borrow_mut().set_builtin(">", &Rc::new(Gt::new()));
        env.borrow_mut().set_builtin(">=", &Rc::new(Ge::new()));
        env.borrow_mut().set_builtin("<", &Rc::new(Lt::new()));
        env.borrow_mut().set_builtin("<=", &Rc::new(Le::new()));

        env.borrow_mut().set_builtin("not", &Rc::new(Not::new()));

        env.borrow_mut().set_builtin("car", &Rc::new(Car::new()));
        env.borrow_mut().set_builtin("cdr", &Rc::new(Cdr::new()));

        env.borrow_mut().set_builtin("list", &Rc::new(List::new()));
        env.borrow_mut()
            .set_builtin("sequence", &Rc::new(Sequence::new()));
        env.borrow_mut().set_builtin("head", &Rc::new(Head::new()));
        env.borrow_mut().set_builtin("tail", &Rc::new(Tail::new()));
        env.borrow_mut().set_builtin("cons", &Rc::new(Cons::new()));
        env.borrow_mut()
            .set_builtin("concat", &Rc::new(Concat::new()));
        env.borrow_mut()
            .set_builtin("filter", &Rc::new(Filter::new()));
        env.borrow_mut().set_builtin("map", &Rc::new(Map::new()));
        env.borrow_mut().set_builtin("drop", &Rc::new(Drop::new()));
        env.borrow_mut()
            .set_builtin("drop-while", &Rc::new(DropWhile::new()));
        env.borrow_mut().set_builtin("take", &Rc::new(Take::new()));
        env.borrow_mut()
            .set_builtin("take-while", &Rc::new(TakeWhile::new()));
        env.borrow_mut()
            .set_builtin("list-ref", &Rc::new(ListRef::new()));
        env.borrow_mut()
            .set_builtin("list-set!", &Rc::new(ListSetBang::new()));

        env.borrow_mut()
            .set_builtin("iterator", &Rc::new(Iterator::new()));

        env.borrow_mut()
            .set_builtin("count", &Rc::new(Count::new()));
        env.borrow_mut()
            .set_builtin("empty?", &Rc::new(IsEmpty::new()));

        env.borrow_mut()
            .set_builtin("str-sub", &Rc::new(StrSub::new()));
        env.borrow_mut()
            .set_builtin("str-replace", &Rc::new(StrReplace::new()));
        env.borrow_mut()
            .set_builtin("str-concat", &Rc::new(StrConcat::new()));
        env.borrow_mut()
            .set_builtin("str-upper", &Rc::new(StrUpper::new()));
        env.borrow_mut()
            .set_builtin("str-lower", &Rc::new(StrLower::new()));
        env.borrow_mut()
            .set_builtin("str->int", &Rc::new(StrToInt::new()));
        env.borrow_mut()
            .set_builtin("str->real", &Rc::new(StrToReal::new()));

        env.borrow_mut()
            .set_builtin("equal?", &Rc::new(IsEqual::new()));

        let input = env.borrow().get_input().clone();
        env.borrow_mut()
            .set_builtin("read-line", &Rc::new(ReadLine::new(&input)));

        Self::init_output_builtins(env);

        Self::init_require_builtin(env);

        env.borrow_mut()
            .set_builtin("struct-get", &Rc::new(StructGet::new()));
        env.borrow_mut()
            .set_builtin("struct-set!", &Rc::new(StructSet::new()));
        env.borrow_mut()
            .set_builtin("create-hash-table", &Rc::new(CreateHashTable::new()));
        env.borrow_mut()
            .set_builtin("hash-get", &Rc::new(HashGet::new()));
        env.borrow_mut()
            .set_builtin("hash-set!", &Rc::new(HashSetBang::new()));
    }

    fn init_output_builtins(env: &EnvironmentRef) {
        let output = env.borrow().get_output().clone();
        env.borrow_mut()
            .set_builtin("write", &Rc::new(Write_::new(&output)));
        env.borrow_mut()
            .set_builtin("writeln", &Rc::new(WriteLn::new(&output)));
        env.borrow_mut()
            .set_builtin("display", &Rc::new(Display_::new(&output)));
        env.borrow_mut()
            .set_builtin("displayln", &Rc::new(DisplayLn::new(&output)));
    }

    fn init_require_builtin(env: &EnvironmentRef) {
        env.borrow_mut()
            .set_builtin("require", &Rc::new(RequireFn::new(env)));
    }

    pub fn set_builtin<T: Callable + 'static>(&mut self, name: &str, function: &Rc<T>) {
        self.set_unowned(
            name.to_string(),
            new_valueref(BuiltInFunctionValue {
                name: name.to_string(),
                function: function.clone(),
            }),
        );
    }
}

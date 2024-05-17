use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::vec;

use crate::interpreter::environment::Environment;
use crate::interpreter::module_mgmt::extension::{new_extension, new_extension_dir, ExtensionRef};
use crate::interpreter::module_mgmt::ExtensionModule;
use crate::interpreter::stdlib::load_module_code;
use crate::interpreter::stream::StreamValue;
use crate::interpreter::{
    borrow_value, downcast_value, error, new_valueref, BoolValue, BuiltInFunctionValue, Callable,
    EvalResult, IntValue, InterpreterError, LambdaValue, PairValue, ValueRef, ValueType,
    VectorValue,
};

use super::list::{Car, Cdr};
use super::string::{StrConcat, StrSub};
use super::vector::{VecConcat, VecCons, VecHead, VecTail};

pub fn create_seq_collection_extension(
    vector_ext: &ExtensionRef,
    list_ext: &ExtensionRef,
    string_ext: &ExtensionRef,
    stream_ext: &ExtensionRef,
) -> ExtensionRef {
    let core_env = Environment::new_ref();
    let mut env = Environment::with_parent(&core_env);

    env.set_callable("head", &Rc::new(Head::new()));
    env.set_callable("tail", &Rc::new(Tail::new()));
    env.set_callable("cons", &Rc::new(Cons::new()));
    env.set_callable("concat", &Rc::new(Concat::new()));
    env.set_callable("filter", &Rc::new(Filter::new(list_ext)));
    env.set_callable("map", &Rc::new(Map::new()));

    let collection_env = Rc::new(RefCell::new(env));

    let deps = new_extension_dir("deps");
    for dep in vec![vector_ext, list_ext, string_ext, stream_ext] {
        deps.borrow_mut().add_extension(dep);
    }

    let mut search_dirs = collection_env.borrow().get_module_search_dirs();
    search_dirs.push(deps);
    Environment::set_module_search_dirs(&collection_env, &search_dirs);

    let values =
        load_module_code(&collection_env, include_str!("seqcol.boli")).unwrap_or(HashMap::new());

    new_extension("seqcol", values)
}

struct Head {}

impl Head {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Head {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("head function expects exactly one argument");
        }

        let value_type = args[0].borrow().get_type();

        match value_type {
            ValueType::Vector => VecHead::new().call(args),
            ValueType::Pair => Car::new().call(args),
            ValueType::Stream => {
                let sequence = args[0].clone();
                let sequence = borrow_value(&sequence);
                let mut sequence = downcast_value::<StreamValue>(&sequence).unwrap().clone();
                match sequence.next() {
                    Some(head) => Ok(head),
                    None => error("head function expects a non-empty stream "),
                }
            }
            ValueType::Str => {
                let start = new_valueref(IntValue { value: 0 });
                let length = new_valueref(IntValue { value: 1 });
                let args = vec![args[0].clone(), start, length];
                StrSub::new().call(&args)
            }
            _ => error("head function expects a non empty sequential collection"),
        }
    }
}

struct Tail {}

impl Tail {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Tail {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("tail function expects exactly one argument");
        }

        let value_type = args[0].borrow().get_type();

        match value_type {
            ValueType::Vector => VecTail::new().call(args),
            ValueType::Pair => Cdr::new().call(args),
            ValueType::Stream => {
                let sequence = args[0].clone();
                let sequence = borrow_value(&sequence);
                let mut sequence = downcast_value::<StreamValue>(&sequence).unwrap().clone();
                sequence.next();
                Ok(new_valueref(sequence))
            }
            ValueType::Str => {
                let start = new_valueref(IntValue { value: 1 });
                let args = vec![args[0].clone(), start];
                StrSub::new().call(&args)
            }
            _ => error("tail function expects a sequential collection"),
        }
    }
}

struct Cons {}

impl Cons {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Cons {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("cons function expects exactly two arguments");
        }

        let second_arg = args[1].borrow();

        match second_arg.get_type() {
            ValueType::Vector => VecCons::new().call(args),
            _ => Ok(new_valueref(PairValue {
                left: args[0].clone(),
                right: args[1].clone(),
            })),
        }
    }
}

pub struct Concat {}

impl Concat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Concat {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let mut unique_type: Option<ValueType> = None;

        for arg in args {
            let arg_type = arg.borrow().get_type();
            if let Some(prev_type) = unique_type {
                if arg_type != prev_type {
                    return error("concat function expects all arguments to be of the same type");
                }
            } else {
                unique_type = Some(arg_type);
            }
        }

        if unique_type.is_none() {
            return error("concat function expects at least one argument");
        }
        let unique_type = unique_type.unwrap();

        match unique_type {
            ValueType::Vector => VecConcat::new().call(args),
            ValueType::Pair => {
                let all_lists = args.iter().all(|arg| {
                    let pair = arg.borrow();
                    let pair = pair.as_any().downcast_ref::<PairValue>().unwrap();
                    pair.is_list()
                });
                if !all_lists {
                    return error("concat function expects all arguments to be lists");
                }
                let result: Option<ValueRef> = args.iter().fold(None, |acc, arg| {
                    if let Some(acc) = acc {
                        let acc_pair = acc.borrow();
                        let acc_pair = acc_pair.as_any().downcast_ref::<PairValue>().unwrap();
                        Some(acc_pair.concat(&arg))
                    } else {
                        Some(arg.clone())
                    }
                });
                result.ok_or(InterpreterError::new(
                    "concat function expects at least one argument",
                ))
            }
            ValueType::Str => StrConcat::new().call(args),
            _ => error("concat function expects arguments to be vectors, lists, or strings"),
        }
    }
}

pub struct Filter {
    list_filter: ValueRef,
}

impl Filter {
    pub fn new(list_ext: &ExtensionRef) -> Self {
        let list_values: Ref<dyn ExtensionModule> = list_ext.borrow();
        let list_values = list_values.get_values();
        Self {
            list_filter: list_values.get("list-filter").unwrap().clone(),
        }
    }
}

impl Callable for Filter {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("filter function expects exactly two arguments");
        }

        let value_type = args[1].borrow().get_type();

        match value_type {
            ValueType::Vector => {
                let arg0 = &borrow_value(&args[0]);
                let predicate: &dyn Callable =
                    match arg0.get_type() {
                        ValueType::BuiltInFunction => {
                            downcast_value::<BuiltInFunctionValue>(arg0).unwrap()
                        }
                        ValueType::Lambda => downcast_value::<LambdaValue>(arg0).unwrap(),
                        _ => return error(
                            "filter function expects a predicate function as the first argument",
                        ),
                    };

                let arg1 = &borrow_value(&args[1]);
                let list = downcast_value::<VectorValue>(arg1).unwrap();
                let mut elements = Vec::new();
                for elem in &list.elements {
                    let result = predicate.call(&vec![elem.clone()])?;
                    let result = borrow_value(&result);
                    match result.get_type() {
                        ValueType::Bool => {
                            if downcast_value::<BoolValue>(&result).unwrap().value {
                                elements.push(elem.clone());
                            }
                        }
                        _ => {
                            return error(
                                "filter function expects a predicate that returns a boolean value",
                            )
                        }
                    }
                }
                Ok(new_valueref(VectorValue { elements }) as ValueRef)
            }
            ValueType::Pair => {
                let filter = &borrow_value(&self.list_filter);
                let filter = downcast_value::<LambdaValue>(filter).unwrap();
                filter.call(args)
            }
            ValueType::Nil => Ok(args[1].clone()),
            ValueType::Stream => {
                let filtered = StreamValue::new_filtered(args[0].clone(), args[1].clone())?;
                Ok(new_valueref(filtered))
            }
            _ => {
                return error(
                    "filter function expects a vector, list or stream as the second argument",
                )
            }
        }
    }
}

struct Map {}

impl Map {
    fn new() -> Self {
        Self {}
    }
}

impl Callable for Map {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() < 2 {
            return error("map function expects at least two arguments");
        }

        let mut type_stats: HashMap<ValueType, usize> = HashMap::new();
        for arg in args.iter().skip(1) {
            let arg = &borrow_value(&arg);
            let arg_type = arg.get_type();

            if !matches!(
                arg_type,
                ValueType::Vector | ValueType::Pair | ValueType::Nil | ValueType::Stream
            ) {
                return error(
                    "map function expects a vector, list, string, or stream as arguments",
                );
            }

            let count = type_stats.get(&arg_type).unwrap_or(&0) + 1;
            type_stats.insert(arg_type, count);
        }

        let total_count: usize = type_stats.values().sum();

        if &total_count == type_stats.get(&ValueType::Stream).unwrap_or(&0) {
            return Ok(new_valueref(StreamValue::new_mapped(
                args[0].clone(),
                args[1..].to_vec(),
            )?));
        }

        if type_stats.contains_key(&ValueType::Nil) {
            return Ok(new_valueref(VectorValue {
                elements: Vec::new(),
            }));
        }

        let arg0 = &borrow_value(&args[0]);
        let function: &dyn Callable = match arg0.get_type() {
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(arg0).unwrap(),
            ValueType::Lambda => downcast_value::<LambdaValue>(arg0).unwrap(),
            _ => return error("map function expects a function as the first argument"),
        };

        let mut values = Vec::new();
        let mut cols = args
            .iter()
            .skip(1)
            .map(|arg| arg.clone())
            .collect::<Vec<ValueRef>>();
        let mut done = false;

        loop {
            let mut function_args = Vec::new();
            let mut next_cols = Vec::new();

            for col in cols.iter() {
                let head_tail_opt = match col.borrow().get_type() {
                    ValueType::Vector => {
                        let vector = borrow_value(col);
                        let vector = downcast_value::<VectorValue>(&vector).unwrap();
                        if !vector.elements.is_empty() {
                            Some((
                                VecHead::new().call(&vec![col.clone()]).unwrap(),
                                VecTail::new().call(&vec![col.clone()]).unwrap(),
                            ))
                        } else {
                            None
                        }
                    }
                    ValueType::Pair => {
                        let pair = borrow_value(col);
                        let pair = downcast_value::<PairValue>(&pair).unwrap();
                        Some((pair.left.clone(), pair.right.clone()))
                    }
                    ValueType::Stream => {
                        let mut stream = downcast_value::<StreamValue>(&col.borrow())
                            .unwrap()
                            .clone();
                        match stream.next() {
                            Some(head) => {
                                let tail = StreamValue::new_dropped(
                                    new_valueref(IntValue { value: 1 }),
                                    col.clone(),
                                )
                                .unwrap();
                                Some((head, new_valueref(tail)))
                            }
                            None => None,
                        }
                    }
                    ValueType::Nil => None,
                    _ => unreachable!(),
                };

                if let Some((head, tail)) = head_tail_opt {
                    function_args.push(head);
                    next_cols.push(tail);
                } else {
                    done = true;
                    break;
                }
            }

            if done {
                break;
            }

            let value = function.call(&function_args)?;
            values.push(value);

            cols = next_cols;
        }

        Ok(new_valueref(VectorValue { elements: values }))
    }
}

use std::fmt::Formatter;

use crate::interpreter::misc_functions::is_truthy;

use super::*;

#[derive(Debug, Clone)]
pub struct LazyListValue {
    iterator: LazyIterator,
    actions: Vec<Action>,
}

impl LazyListValue {
    pub fn new(start: &ValueRef, next_function: &ValueRef) -> Result<Self, InterpreterError> {
        let iterator = LazyIterator::new(start, next_function)?;
        Ok(Self {
            iterator,
            actions: Vec::new(),
        })
    }

    pub fn filter(&self, predicate: &ValueRef) -> Result<Self, InterpreterError> {
        let mut new_list = self.clone();
        new_list.actions.push(Action::new_filter(predicate)?);
        Ok(new_list)
    }

    pub fn map(&self, function: &ValueRef) -> Result<Self, InterpreterError> {
        let mut new_list = self.clone();
        new_list.actions.push(Action::new_map(function)?);
        Ok(new_list)
    }

    pub fn drop(&self, count: &ValueRef) -> Result<Self, InterpreterError> {
        let mut new_list = self.clone();
        new_list.actions.push(Action::new_drop(count)?);
        Ok(new_list)
    }

    pub fn drop_while(&self, predicate: &ValueRef) -> Result<Self, InterpreterError> {
        let mut new_list = self.clone();
        new_list.actions.push(Action::new_drop_while(predicate)?);
        Ok(new_list)
    }

    pub fn take(&mut self, n: &ValueRef) -> EvalResult {
        if n.borrow().get_type() != ValueType::Int {
            return Err(InterpreterError::new("Take count must be an integer"));
        }
        let n = &borrow_value(n);
        let n = downcast_value::<IntValue>(n).unwrap().value as usize;
        let mut elements: Vec<ValueRef> = Vec::new();

        let mut iter = self.iterator.clone();
        self.reset_actions();

        while !iter.is_done() && elements.len() < n {
            let mut item = iter.next()?;
            let mut is_ok = true;
            for action in &mut self.actions {
                let (new_item, should_continue) = action.perform(&item);
                item = new_item;
                if !should_continue {
                    is_ok = false;
                    break;
                }
            }
            if is_ok {
                elements.push(item);
            }
        }

        Ok(new_valueref(ListValue { elements }))
    }

    pub fn take_while(&mut self, predicate: &ValueRef) -> EvalResult {
        let mut elements: Vec<ValueRef> = Vec::new();

        let mut iter = self.iterator.clone();
        self.reset_actions();

        while !iter.is_done() {
            let mut item = iter.next()?;
            let mut is_ok = true;
            for action in &mut self.actions {
                let (new_item, should_continue) = action.perform(&item);
                item = new_item;
                if !should_continue {
                    is_ok = false;
                    break;
                }
            }
            if is_ok {
                let predicate = predicate.borrow();
                let callable: &dyn Callable = match predicate.get_type() {
                    ValueType::Lambda => downcast_value::<LambdaValue>(&predicate).unwrap(),
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(&predicate).unwrap()
                    }
                    _ => return Err(InterpreterError::new("Predicate must be a function")),
                };

                let call_result = callable.call(&vec![item.clone()]).unwrap();
                if is_truthy(&call_result) {
                    elements.push(item);
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(new_valueref(ListValue { elements }))
    }

    fn reset_actions(&mut self) {
        self.actions.iter_mut().for_each(|action| action.reset());
    }
}

impl Value for LazyListValue {
    fn get_type(&self) -> ValueType {
        ValueType::LazyList
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for LazyListValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<lazy-list>")
    }
}

#[derive(Debug, Clone)]
struct LazyIterator {
    start: ValueRef,
    next_function: ValueRef,
}

impl LazyIterator {
    fn new(start: &ValueRef, next_function: &ValueRef) -> Result<Self, InterpreterError> {
        if is_callable(&next_function) {
            Ok(Self {
                start: start.clone(),
                next_function: next_function.clone(),
            })
        } else {
            Err(InterpreterError::new("Next function must be a function"))
        }
    }

    fn next(&mut self) -> Result<ValueRef, InterpreterError> {
        let callee = self.next_function.clone();
        let callee = callee.borrow();
        let callee_type = callee.get_type();

        let callable: &dyn Callable = match callee_type {
            ValueType::Lambda => downcast_value::<LambdaValue>(&callee).unwrap(),
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(&callee).unwrap(),
            _ => unreachable!(),
        };
        let ret = self.start.clone();

        self.start = callable.call(&vec![self.start.clone()])?;

        Ok(ret)
    }

    fn is_done(&self) -> bool {
        self.start.borrow().get_type() == ValueType::Nil
    }
}

fn is_callable(value: &ValueRef) -> bool {
    matches!(
        value.borrow().get_type(),
        ValueType::Lambda | ValueType::BuiltInFunction
    )
}

#[derive(Clone, Debug)]
enum Action {
    Filter { predicate: ValueRef },
    Map { function: ValueRef },
    Drop { count: ValueRef, cnt_dropped: usize },
    DropWhile { predicate: ValueRef, is_done: bool },
}

impl Action {
    fn new_filter(predicate: &ValueRef) -> Result<Self, InterpreterError> {
        if is_callable(predicate) {
            Ok(Self::Filter {
                predicate: predicate.clone(),
            })
        } else {
            Err(InterpreterError::new("Predicate must be a function"))
        }
    }

    fn new_map(function: &ValueRef) -> Result<Self, InterpreterError> {
        if is_callable(function) {
            Ok(Self::Map {
                function: function.clone(),
            })
        } else {
            Err(InterpreterError::new(
                "Map action argument must be a function",
            ))
        }
    }

    fn new_drop(count: &ValueRef) -> Result<Self, InterpreterError> {
        match count.borrow().get_type() {
            ValueType::Int => Ok(Self::Drop {
                count: count.clone(),
                cnt_dropped: 0,
            }),
            _ => Err(InterpreterError::new("Drop count must be an integer")),
        }
    }

    fn new_drop_while(predicate: &ValueRef) -> Result<Self, InterpreterError> {
        if is_callable(predicate) {
            Ok(Self::DropWhile {
                predicate: predicate.clone(),
                is_done: false,
            })
        } else {
            Err(InterpreterError::new(
                "DropWhile predicate must be a function",
            ))
        }
    }

    fn perform(&mut self, item: &ValueRef) -> (ValueRef, bool) {
        match self {
            Self::Filter { predicate } => {
                let predicate = predicate.borrow();
                let callable: &dyn Callable = match predicate.get_type() {
                    ValueType::Lambda => downcast_value::<LambdaValue>(&predicate).unwrap(),
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(&predicate).unwrap()
                    }
                    _ => unreachable!(),
                };

                let call_result = callable.call(&vec![item.clone()]).unwrap();
                (item.clone(), is_truthy(&call_result))
            }
            Self::Map { function } => {
                let function = function.borrow();
                let callable: &dyn Callable = match function.get_type() {
                    ValueType::Lambda => downcast_value::<LambdaValue>(&function).unwrap(),
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(&function).unwrap()
                    }
                    _ => unreachable!(),
                };

                (callable.call(&vec![item.clone()]).unwrap(), true)
            }
            Self::Drop { count, cnt_dropped } => {
                let n = &borrow_value(count);
                let n = downcast_value::<IntValue>(n).unwrap().value as usize;
                if *cnt_dropped < n {
                    *cnt_dropped += 1;
                    (item.clone(), false)
                } else {
                    (item.clone(), true)
                }
            }
            Self::DropWhile { predicate, is_done } => {
                if !*is_done {
                    let predicate = predicate.borrow();
                    let callable: &dyn Callable = match predicate.get_type() {
                        ValueType::Lambda => downcast_value::<LambdaValue>(&predicate).unwrap(),
                        ValueType::BuiltInFunction => {
                            downcast_value::<BuiltInFunctionValue>(&predicate).unwrap()
                        }
                        _ => unreachable!(),
                    };

                    let call_result = callable.call(&vec![item.clone()]).unwrap();
                    *is_done = !is_truthy(&call_result);
                }

                (item.clone(), *is_done)
            }
        }
    }

    fn reset(&mut self) {
        match self {
            Self::Drop { cnt_dropped, .. } => *cnt_dropped = 0,
            Self::DropWhile { is_done, .. } => *is_done = false,
            _ => (),
        }
    }
}

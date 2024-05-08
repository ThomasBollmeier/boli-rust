use crate::interpreter::misc_functions::is_truthy;

use super::*;

#[derive(Debug, Clone)]
pub enum Sequence {
    List {
        list: ValueRef,
        index: usize,
    },
    Iterator {
        start: ValueRef,
        next_func: ValueRef,
    },
    Filtered {
        predicate_func: ValueRef,
        sequence: ValueRef,
    },
    Mapped {
        map_func: ValueRef,
        sequences: Vec<ValueRef>,
    },
}

impl Sequence {
    pub fn new_list(list: ValueRef) -> Result<Self, InterpreterError> {
        if list.borrow().get_type() != ValueType::List {
            return Err(InterpreterError::new(
                "Expected list value to create sequence.",
            ));
        }

        Ok(Self::List { list, index: 0 })
    }

    pub fn new_iterator(next_func: ValueRef, start: ValueRef) -> Result<Self, InterpreterError> {
        if !matches!(
            next_func.borrow().get_type(),
            ValueType::BuiltInFunction | ValueType::Lambda
        ) {
            return Err(InterpreterError::new(
                "Expected function value to create sequence.",
            ));
        }

        Ok(Self::Iterator {
            start: start.clone(),
            next_func: next_func.clone(),
        })
    }

    pub fn new_filtered(
        predicate_func: ValueRef,
        sequence: ValueRef,
    ) -> Result<Self, InterpreterError> {
        if !matches!(
            predicate_func.borrow().get_type(),
            ValueType::BuiltInFunction | ValueType::Lambda
        ) {
            return Err(InterpreterError::new(
                "Filtered sequence requires a function as predicate.",
            ));
        }

        if sequence.borrow().get_type() != ValueType::Sequence {
            return Err(InterpreterError::new(
                "Filtered sequence requires a sequence.",
            ));
        }

        Ok(Self::Filtered {
            predicate_func: predicate_func.clone(),
            sequence: sequence.clone(),
        })
    }

    pub fn new_mapped(
        map_func: ValueRef,
        sequences: Vec<ValueRef>,
    ) -> Result<Self, InterpreterError> {
        if !matches!(
            map_func.borrow().get_type(),
            ValueType::BuiltInFunction | ValueType::Lambda
        ) {
            return Err(InterpreterError::new(
                "Mapped sequence requires a function as mapper.",
            ));
        }

        if !sequences
            .iter()
            .all(|sequence| matches!(sequence.borrow().get_type(), ValueType::Sequence))
        {
            return Err(InterpreterError::new(
                "Mapped sequence requires a sequence.",
            ));
        }

        Ok(Self::Mapped {
            map_func: map_func.clone(),
            sequences: sequences.clone(),
        })
    }

    pub fn next(&mut self) -> Option<ValueRef> {
        match self {
            Self::List { list, index } => {
                let list = &borrow_value(list);
                let list = downcast_value::<ListValue>(list).unwrap();
                if *index < list.elements.len() {
                    let value = list.elements.get(*index).unwrap();
                    *index += 1;
                    Some(value.clone())
                } else {
                    None
                }
            }
            Self::Iterator { next_func, start } => {
                let result = match borrow_value(start).get_type() {
                    ValueType::Nil => return None,
                    _ => Some(start.clone()),
                };

                let next_func = &borrow_value(next_func);
                let callable: &dyn Callable = match next_func.get_type() {
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(next_func).unwrap()
                    }
                    ValueType::Lambda => downcast_value::<LambdaValue>(next_func).unwrap(),
                    _ => unreachable!(),
                };

                *start = callable
                    .call(&vec![start.clone()])
                    .unwrap_or(new_valueref(NilValue {}));

                result
            }
            Self::Filtered {
                predicate_func,
                sequence,
            } => {
                let pred = &borrow_value(&predicate_func);
                let pred: &dyn Callable = match pred.get_type() {
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(pred).unwrap()
                    }
                    ValueType::Lambda => downcast_value::<LambdaValue>(pred).unwrap(),
                    _ => unreachable!(),
                };

                let mut seq = borrow_mut_value(sequence);
                let seq = seq.as_any_mut().downcast_mut::<Sequence>().unwrap();

                loop {
                    if let Some(value) = seq.next() {
                        let result = pred.call(&vec![value.clone()]);
                        if result.is_err() {
                            return None;
                        }
                        let result = result.unwrap();
                        if is_truthy(&result) {
                            return Some(value);
                        }
                    } else {
                        return None;
                    }
                }
            }
            Self::Mapped {
                map_func,
                sequences,
            } => {
                let func = &borrow_value(&map_func);
                let func: &dyn Callable = match func.get_type() {
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(func).unwrap()
                    }
                    ValueType::Lambda => downcast_value::<LambdaValue>(func).unwrap(),
                    _ => unreachable!(),
                };

                let mut args = Vec::new();
                for sequence in sequences {
                    if let Some(value) = borrow_mut_value(sequence)
                        .as_any_mut()
                        .downcast_mut::<Sequence>()
                        .unwrap()
                        .next()
                    {
                        args.push(value);
                    } else {
                        return None;
                    }
                }

                func.call(&args).ok()
            }
        }
    }
}

impl Value for Sequence {
    fn get_type(&self) -> ValueType {
        ValueType::Sequence
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for Sequence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<sequence>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn take(n: usize, sequence: &Sequence) -> ListValue {
        let mut seq = sequence.clone();
        let mut elements = Vec::new();
        for _ in 0..n {
            if let Some(value) = seq.next() {
                elements.push(value);
            } else {
                break;
            }
        }
        ListValue { elements }
    }

    #[test]
    fn test_list() {
        let list = new_valueref(ListValue {
            elements: vec![
                new_valueref(IntValue { value: 1 }),
                new_valueref(IntValue { value: 2 }),
                new_valueref(IntValue { value: 3 }),
            ],
        });

        let mut sequence = Sequence::new_list(list).unwrap();

        assert_eq!(take(10, &mut sequence).to_string(), "(list 1 2 3)");
    }

    #[test]
    fn test_iterator() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let mut sequence = Sequence::new_iterator(next_func, start).unwrap();

        assert_eq!(
            take(10, &mut sequence).to_string(),
            "(list 0 1 2 3 4 5 6 7 8 9)"
        );

        assert_eq!(
            take(10, &mut sequence).to_string(),
            "(list 0 1 2 3 4 5 6 7 8 9)"
        );
    }

    #[test]
    fn test_filtered() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let numbers = Sequence::new_iterator(next_func, start).unwrap();

        let predicate_func = interpreter.eval("(λ (n) (= (% n 2) 0))").unwrap();
        let even_numbers =
            Sequence::new_filtered(predicate_func, new_valueref(numbers.clone())).unwrap();

        assert_eq!(take(10, &numbers).to_string(), "(list 0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(
            take(10, &even_numbers).to_string(),
            "(list 0 2 4 6 8 10 12 14 16 18)"
        );
    }

    #[test]
    fn test_mapped() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let numbers = Sequence::new_iterator(next_func, start).unwrap();

        let map_func = interpreter.eval("(λ (i j) (list i (* j j)))").unwrap();
        let squared_numbers = Sequence::new_mapped(
            map_func,
            vec![new_valueref(numbers.clone()), new_valueref(numbers.clone())],
        )
        .unwrap();

        assert_eq!(take(10, &numbers).to_string(), "(list 0 1 2 3 4 5 6 7 8 9)");
        assert_eq!(
            take(4, &squared_numbers).to_string(),
            "(list (list 0 0) (list 1 1) (list 2 4) (list 3 9))"
        );
    }
}

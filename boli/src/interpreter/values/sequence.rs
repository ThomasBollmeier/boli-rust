use crate::interpreter::misc_functions::is_truthy;

use super::*;

#[derive(Debug)]
pub enum SequenceValue {
    List {
        list: ValueRef,
        index: usize,
    },
    Iterator {
        start: ValueRef,
        current: ValueRef,
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
    Dropped {
        n: ValueRef,
        sequence: ValueRef,
        initial: bool,
    },
    DroppedWhile {
        predicate_func: ValueRef,
        sequence: ValueRef,
        initial: bool,
    },
}

impl SequenceValue {
    pub fn new_list(list: ValueRef) -> Result<Self, InterpreterError> {
        if list.borrow().get_type() != ValueType::Vector {
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
            current: start.clone(),
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

    pub fn new_dropped(n: ValueRef, sequence: ValueRef) -> Result<Self, InterpreterError> {
        if n.borrow().get_type() != ValueType::Int {
            return Err(InterpreterError::new(
                "Dropped sequence requires an integer.",
            ));
        }

        if sequence.borrow().get_type() != ValueType::Sequence {
            return Err(InterpreterError::new(
                "Dropped sequence requires a sequence.",
            ));
        }

        Ok(Self::Dropped {
            n: n.clone(),
            sequence: sequence.clone(),
            initial: true,
        })
    }

    pub fn new_dropped_while(
        predicate_func: ValueRef,
        sequence: ValueRef,
    ) -> Result<Self, InterpreterError> {
        if !matches!(
            predicate_func.borrow().get_type(),
            ValueType::BuiltInFunction | ValueType::Lambda
        ) {
            return Err(InterpreterError::new(
                "DroppedWhile sequence requires a function as predicate.",
            ));
        }

        if sequence.borrow().get_type() != ValueType::Sequence {
            return Err(InterpreterError::new(
                "DroppedWhile sequence requires a sequence.",
            ));
        }

        Ok(Self::DroppedWhile {
            predicate_func: predicate_func.clone(),
            sequence: sequence.clone(),
            initial: true,
        })
    }

    pub fn next(&mut self) -> Option<ValueRef> {
        match self {
            Self::List { list, index } => {
                let list = &borrow_value(list);
                let list = downcast_value::<VectorValue>(list).unwrap();
                if *index < list.elements.len() {
                    let value = list.elements.get(*index).unwrap();
                    *index += 1;
                    Some(value.clone())
                } else {
                    None
                }
            }
            Self::Iterator {
                start: _start,
                current,
                next_func,
            } => {
                let result = match borrow_value(current).get_type() {
                    ValueType::Nil => return None,
                    _ => Some(current.clone()),
                };

                let next_func = &borrow_value(next_func);
                let callable: &dyn Callable = match next_func.get_type() {
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(next_func).unwrap()
                    }
                    ValueType::Lambda => downcast_value::<LambdaValue>(next_func).unwrap(),
                    _ => unreachable!(),
                };

                *current = callable
                    .call(&vec![current.clone()])
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
                let seq = seq.as_any_mut().downcast_mut::<SequenceValue>().unwrap();

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
                        .downcast_mut::<SequenceValue>()
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
            Self::Dropped {
                n,
                sequence,
                initial,
            } => {
                let mut sequence = borrow_mut_value(sequence);
                let sequence = sequence
                    .as_any_mut()
                    .downcast_mut::<SequenceValue>()
                    .unwrap();

                if *initial {
                    let n = borrow_value(n);
                    let n = downcast_value::<IntValue>(&n).unwrap().value;

                    for _ in 0..n {
                        if sequence.next().is_none() {
                            return None;
                        }
                    }
                    *initial = false;
                }

                sequence.next()
            }
            Self::DroppedWhile {
                predicate_func,
                sequence,
                initial,
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
                let seq = seq.as_any_mut().downcast_mut::<SequenceValue>().unwrap();

                if *initial {
                    loop {
                        if let Some(value) = seq.next() {
                            let result = pred.call(&vec![value.clone()]);
                            if result.is_err() {
                                return None;
                            }
                            let result = result.unwrap();
                            if !is_truthy(&result) {
                                *initial = false;
                                return Some(value);
                            }
                        } else {
                            return None;
                        }
                    }
                }

                seq.next()
            }
        }
    }

    fn clone_sequence(value: &ValueRef) -> ValueRef {
        let value = borrow_value(&value);
        let value = value.as_any().downcast_ref::<SequenceValue>().unwrap();
        new_valueref(value.clone())
    }
}

impl Clone for SequenceValue {
    fn clone(&self) -> Self {
        match self {
            Self::List { list, index } => Self::List {
                list: list.clone(),
                index: index.clone(),
            },
            Self::Iterator {
                start,
                current,
                next_func,
            } => Self::Iterator {
                start: start.clone(),
                current: current.clone(),
                next_func: next_func.clone(),
            },
            Self::Filtered {
                predicate_func,
                sequence,
            } => Self::Filtered {
                predicate_func: predicate_func.clone(),
                sequence: Self::clone_sequence(sequence),
            },
            Self::Mapped {
                map_func,
                sequences,
            } => Self::Mapped {
                map_func: map_func.clone(),
                sequences: sequences.iter().map(Self::clone_sequence).collect(),
            },
            Self::Dropped {
                n,
                sequence,
                initial: _initial,
            } => Self::Dropped {
                n: n.clone(),
                sequence: Self::clone_sequence(sequence),
                initial: true,
            },
            Self::DroppedWhile {
                predicate_func,
                sequence,
                initial: _initial,
            } => Self::DroppedWhile {
                predicate_func: predicate_func.clone(),
                sequence: Self::clone_sequence(sequence),
                initial: true,
            },
        }
    }
}

impl Value for SequenceValue {
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

impl Display for SequenceValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<sequence>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn take(n: usize, sequence: &SequenceValue) -> VectorValue {
        let mut seq = sequence.clone();
        let mut elements = Vec::new();
        for _ in 0..n {
            if let Some(value) = seq.next() {
                elements.push(value);
            } else {
                break;
            }
        }
        VectorValue { elements }
    }

    #[test]
    fn test_list() {
        let list = new_valueref(VectorValue {
            elements: vec![
                new_valueref(IntValue { value: 1 }),
                new_valueref(IntValue { value: 2 }),
                new_valueref(IntValue { value: 3 }),
            ],
        });

        let mut sequence = SequenceValue::new_list(list).unwrap();

        assert_eq!(take(10, &mut sequence).to_string(), "(vector 1 2 3)");
    }

    #[test]
    fn test_iterator() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let mut sequence = SequenceValue::new_iterator(next_func, start).unwrap();

        assert_eq!(
            take(10, &mut sequence).to_string(),
            "(vector 0 1 2 3 4 5 6 7 8 9)"
        );

        assert_eq!(
            take(10, &mut sequence).to_string(),
            "(vector 0 1 2 3 4 5 6 7 8 9)"
        );
    }

    #[test]
    fn test_filtered() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let numbers = SequenceValue::new_iterator(next_func, start).unwrap();

        let predicate_func = interpreter.eval("(λ (n) (= (% n 2) 0))").unwrap();
        let even_numbers =
            SequenceValue::new_filtered(predicate_func, new_valueref(numbers.clone())).unwrap();

        assert_eq!(
            take(10, &numbers).to_string(),
            "(vector 0 1 2 3 4 5 6 7 8 9)"
        );
        assert_eq!(
            take(10, &even_numbers).to_string(),
            "(vector 0 2 4 6 8 10 12 14 16 18)"
        );
    }

    #[test]
    fn test_mapped() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let numbers = SequenceValue::new_iterator(next_func, start).unwrap();

        let map_func = interpreter.eval("(λ (i j) (i . (* j j)))").unwrap();
        let squared_numbers = SequenceValue::new_mapped(
            map_func,
            vec![new_valueref(numbers.clone()), new_valueref(numbers.clone())],
        )
        .unwrap();

        assert_eq!(
            take(10, &numbers).to_string(),
            "(vector 0 1 2 3 4 5 6 7 8 9)"
        );
        assert_eq!(
            take(4, &squared_numbers).to_string(),
            "(vector (0 . 0) (1 . 1) (2 . 4) (3 . 9))"
        );
    }

    #[test]
    fn test_dropped() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let numbers = SequenceValue::new_iterator(next_func, start).unwrap();

        let dropped = SequenceValue::new_dropped(
            new_valueref(IntValue { value: 5 }),
            new_valueref(numbers.clone()),
        )
        .unwrap();

        assert_eq!(
            take(10, &dropped).to_string(),
            "(vector 5 6 7 8 9 10 11 12 13 14)"
        );
    }

    #[test]
    fn test_dropped_while() {
        let mut interpreter = Interpreter::new();

        let next_func = interpreter.eval("(λ (n) (+ n 1))").unwrap();
        let start = new_valueref(IntValue { value: 0 });

        let numbers = SequenceValue::new_iterator(next_func, start).unwrap();

        let predicate_func = interpreter.eval("(λ (n) (< n 5))").unwrap();
        let dropped =
            SequenceValue::new_dropped_while(predicate_func, new_valueref(numbers.clone()))
                .unwrap();

        assert_eq!(
            take(10, &dropped).to_string(),
            "(vector 5 6 7 8 9 10 11 12 13 14)"
        );
    }
}

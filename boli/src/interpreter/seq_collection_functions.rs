use crate::interpreter::misc_functions::is_truthy;

use super::values::sequence::*;
use super::values::*;

pub struct Vector {}

impl Vector {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Vector {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        Ok(new_valueref(VectorValue {
            elements: args.clone(),
        }))
    }
}

pub struct Sequence {}

impl Sequence {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Sequence {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("sequence function expects one argument");
        }

        if args[0].borrow().get_type() != ValueType::Vector {
            return error("sequence function expects a list as the argument");
        }

        Ok(new_valueref(StreamValue::new_list(args[0].clone())?))
    }
}

pub struct Iterator {}

impl Iterator {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Iterator {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("iterator function expects two arguments");
        }

        let (start, next_function) = (&args[0], &args[1]);
        let iterator = StreamValue::new_iterator(next_function.clone(), start.clone())?;

        Ok(new_valueref(iterator))
    }
}

pub struct Head {}

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
            ValueType::Vector => {
                let list = &borrow_value(&args[0]);
                let list = downcast_value::<VectorValue>(list).unwrap();
                if list.elements.is_empty() {
                    return error("head function expects a non-empty list");
                }
                Ok(list.elements[0].clone())
            }
            ValueType::Stream => {
                let sequence = args[0].clone();
                let sequence = borrow_value(&sequence);
                let mut sequence = downcast_value::<StreamValue>(&sequence).unwrap().clone();
                match sequence.next() {
                    Some(head) => Ok(head),
                    None => error("head function expects a non-empty sequence"),
                }
            }
            ValueType::Pair => {
                let pair = &borrow_value(&args[0]);
                let pair = downcast_value::<PairValue>(pair).unwrap();
                Ok(pair.left.clone())
            }
            _ => error("head function expects a list"),
        }
    }
}

pub struct Tail {}

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
            ValueType::Vector => {
                let list = &borrow_value(&args[0]);
                let list = downcast_value::<VectorValue>(list).unwrap();
                if list.elements.is_empty() {
                    return error("tail function expects a non-empty list");
                }
                Ok(new_valueref(VectorValue {
                    elements: list.elements[1..].to_vec(),
                }))
            }
            ValueType::Stream => {
                let sequence = args[0].clone();
                let sequence = borrow_value(&sequence);
                let mut sequence = downcast_value::<StreamValue>(&sequence).unwrap().clone();
                sequence.next();
                Ok(new_valueref(sequence))
            }
            ValueType::Pair => {
                let pair = &borrow_value(&args[0]);
                let pair = downcast_value::<PairValue>(pair).unwrap();
                Ok(pair.right.clone())
            }
            _ => error("tail function expects a list"),
        }
    }
}

pub struct Cons {}

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
        match second_arg.as_any().downcast_ref::<VectorValue>() {
            Some(list) => {
                let mut elements = vec![args[0].clone()];
                elements.extend(list.elements.clone());

                Ok(new_valueref(VectorValue { elements }) as ValueRef)
            }
            None => Ok(new_valueref(PairValue {
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
        let mut elements = Vec::new();

        for arg in args {
            let arg = arg.borrow();
            match arg.as_any().downcast_ref::<VectorValue>() {
                Some(list) => elements.extend(list.elements.clone()),
                None => return error("concat function expects a list as arguments"),
            }
        }

        Ok(new_valueref(VectorValue { elements }) as ValueRef)
    }
}

pub struct Filter {}

impl Filter {
    pub fn new() -> Self {
        Self {}
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
            ValueType::Stream => {
                let filtered = StreamValue::new_filtered(args[0].clone(), args[1].clone())?;
                Ok(new_valueref(filtered))
            }
            _ => return error("filter function expects a list as the second argument"),
        }
    }
}

pub struct Map {}

impl Map {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Map {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() < 2 {
            return error("map function expects at least two arguments");
        }

        let lists = &args[1..];
        let all_eager = lists.iter().all(|arg| {
            let arg = borrow_value(&arg);
            arg.get_type() == ValueType::Vector
        });

        if all_eager {
            let arg0 = &borrow_value(&args[0]);
            let function: &dyn Callable = match arg0.get_type() {
                ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(arg0).unwrap(),
                ValueType::Lambda => downcast_value::<LambdaValue>(arg0).unwrap(),
                _ => return error("map function expects a function as the first argument"),
            };

            let mut min_size_opt: Option<usize> = None;

            for arg in args.iter().skip(1) {
                let arg = &borrow_value(&arg);
                match arg.get_type() {
                    ValueType::Vector => {
                        let list = downcast_value::<VectorValue>(arg).unwrap();
                        if let Some(min_size) = min_size_opt {
                            if list.elements.len() < min_size {
                                min_size_opt = Some(list.elements.len());
                            }
                        } else {
                            min_size_opt = Some(list.elements.len());
                        }
                    }
                    _ => return error("map function expects a list as arguments"),
                }
            }

            let min_size = min_size_opt.unwrap_or(0);

            let mut elements = Vec::new();

            for i in 0..min_size {
                let mut call_args = Vec::new();
                for arg in args.iter().skip(1) {
                    let arg = &borrow_value(&arg);
                    let list = downcast_value::<VectorValue>(arg).unwrap();
                    call_args.push(list.elements[i].clone());
                }

                let result = function.call(&call_args)?;
                elements.push(result);
            }

            Ok(new_valueref(VectorValue { elements }) as ValueRef)
        } else {
            let map_function = args[0].clone();
            let mut sequences = vec![];
            for lst in lists {
                let value_type = lst.borrow().get_type();
                match value_type {
                    ValueType::Vector => {
                        sequences.push(new_valueref(StreamValue::new_list(lst.clone())?));
                    }
                    ValueType::Stream => {
                        sequences.push(lst.clone());
                    }
                    _ => return error("map function expects a list or a sequence as arguments"),
                }
            }
            let mapped = StreamValue::new_mapped(map_function, sequences)?;

            Ok(new_valueref(mapped))
        }
    }
}

pub struct Drop {}

impl Drop {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Drop {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("drop function expects exactly two arguments");
        }

        let first_type = args[0].borrow().get_type();

        if first_type != ValueType::Int {
            return error("drop function expects an integer as the first argument");
        }

        let second_type = args[1].borrow().get_type();

        match second_type {
            ValueType::Vector => {
                let arg0 = &borrow_value(&args[0]);
                let n = arg0.as_any().downcast_ref::<IntValue>().unwrap().value;

                let arg1 = &borrow_value(&args[1]);
                let list = downcast_value::<VectorValue>(arg1).unwrap();
                let elements = list.elements.iter().skip(n as usize).cloned().collect();

                Ok(new_valueref(VectorValue { elements }))
            }
            ValueType::Stream => {
                let dropped = StreamValue::new_dropped(args[0].clone(), args[1].clone())?;
                Ok(new_valueref(dropped))
            }
            _ => return error("drop function expects a list as the second argument"),
        }
    }
}

pub struct DropWhile {}

impl DropWhile {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for DropWhile {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("drop-while function expects exactly two arguments");
        }

        if !matches!(
            args[0].borrow().get_type(),
            ValueType::BuiltInFunction | ValueType::Lambda
        ) {
            return error("drop-while function expects a function as the first argument");
        }

        let arg1_type = args[1].borrow().get_type();

        match arg1_type {
            ValueType::Vector => {
                let arg0 = &borrow_value(&args[0]);
                let predicate: &dyn Callable = match arg0.get_type() {
                    ValueType::BuiltInFunction => {
                        downcast_value::<BuiltInFunctionValue>(arg0).unwrap()
                    }
                    ValueType::Lambda => downcast_value::<LambdaValue>(arg0).unwrap(),
                    _ => unreachable!(),
                };

                let arg1 = &borrow_value(&args[1]);
                let list = downcast_value::<VectorValue>(arg1).unwrap();
                let mut elements = Vec::new();
                let mut drop = true;
                for elem in &list.elements {
                    if drop {
                        let result = predicate.call(&vec![elem.clone()])?;
                        if !is_truthy(&result) {
                            drop = false;
                            elements.push(elem.clone());
                        }
                    } else {
                        elements.push(elem.clone());
                    }
                }
                Ok(new_valueref(VectorValue { elements }))
            }
            ValueType::Stream => {
                let dropped = StreamValue::new_dropped_while(args[0].clone(), args[1].clone())?;
                Ok(new_valueref(dropped))
            }
            _ => return error("drop-while function expects a list as the second argument"),
        }
    }
}

pub struct Take {}

impl Take {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Take {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("take function expects exactly two arguments");
        }

        let first_type = args[0].borrow().get_type();

        if first_type != ValueType::Int {
            return error("take function expects an integer as the first argument");
        }

        let arg0 = &borrow_value(&args[0]);
        let n = arg0.as_any().downcast_ref::<IntValue>().unwrap().value;

        let second_type = args[1].borrow().get_type();

        match second_type {
            ValueType::Vector => {
                let arg1 = &borrow_value(&args[1]);
                let list = downcast_value::<VectorValue>(arg1).unwrap();
                let elements = list.elements.iter().take(n as usize).cloned().collect();

                Ok(new_valueref(VectorValue { elements }))
            }
            ValueType::Stream => {
                let arg1 = &borrow_value(&args[1]);
                let mut seq = downcast_value::<StreamValue>(arg1).unwrap().clone();
                let mut elements = vec![];
                for _ in 0..n {
                    if let Some(elem) = seq.next() {
                        elements.push(elem);
                    }
                }

                Ok(new_valueref(VectorValue { elements }))
            }
            _ => return error("take function expects a list as the second argument"),
        }
    }
}

pub struct TakeWhile {}

impl TakeWhile {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for TakeWhile {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("take-while function expects exactly two arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let predicate: &dyn Callable = match arg0.get_type() {
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(arg0).unwrap(),
            ValueType::Lambda => downcast_value::<LambdaValue>(arg0).unwrap(),
            _ => return error("take-while function expects a function as the first argument"),
        };

        let arg1_type = &borrow_value(&args[1]).get_type();

        match arg1_type {
            ValueType::Vector => {
                let arg1 = &borrow_value(&args[1]);
                let list = downcast_value::<VectorValue>(arg1).unwrap();
                let mut elements = Vec::new();
                for elem in &list.elements {
                    let result = predicate.call(&vec![elem.clone()])?;
                    if is_truthy(&result) {
                        elements.push(elem.clone());
                    } else {
                        break;
                    }
                }
                Ok(new_valueref(VectorValue { elements }))
            }
            ValueType::Stream => {
                let arg1 = &borrow_value(&args[1]);
                let mut seq = downcast_value::<StreamValue>(arg1).unwrap().clone();
                let mut elements = vec![];
                while let Some(elem) = seq.next() {
                    let result = predicate.call(&vec![elem.clone()])?;
                    if is_truthy(&result) {
                        elements.push(elem);
                    } else {
                        break;
                    }
                }
                Ok(new_valueref(VectorValue { elements }))
            }
            _ => return error("take-while function expects a list as the second argument"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter;

    #[test]
    fn test_head_seq() {
        let code = r#"
        (head (iterator 0 (lambda (x) (+ x 1))))
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "0");
    }

    #[test]
    fn test_tail_seq() {
        let code = r#"
        (head (tail (iterator 0 (lambda (x) (+ x 1)))))
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "1");
    }

    #[test]
    fn test_filter_seq() {
        let code = r#"
        (def odds (filter (lambda (x) (= (% x 2) 1)) 
                          (iterator 0 (lambda (x) (+ x 1)))))
        (head odds)
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "1");
    }

    #[test]
    fn test_map_seq() {
        let code = r#"
        (def squares (map (lambda (x) (* x x)) 
                          (iterator 2 (lambda (x) (+ x 1)))))
        (head squares)
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "4");
    }

    #[test]
    fn test_drop_seq() {
        let code = r#"
        (def squares (map (lambda (x) (* x x)) 
                          (iterator 0 (lambda (x) (+ x 1)))))
        (head (drop 2 squares))
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "4");
    }

    #[test]
    fn test_drop_while_seq() {
        let code = r#"
        (def squares (map (lambda (x) (* x x)) 
                          (iterator 0 (lambda (x) (+ x 1)))))
        (head (drop-while (λ (n) (< n 50)) squares))
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "64");
    }

    #[test]
    fn test_take_seq() {
        let code = r#"
        (def naturals (iterator 0 (lambda (x) (+ x 1))))
        (take 3 (drop 1 naturals))
        (take 3 naturals)
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "(vector 0 1 2)");
    }

    #[test]
    fn test_take_while_seq() {
        let code = r#"
        (def (next-pair p)
            (let [(a (car p))
                  (b (cdr p))]
                (b . (+ a b))))
        (def fib (map head (iterator (0 . 1) next-pair)))
        (take-while (λ (n) (< n 100)) fib)
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(
            result.borrow().to_string(),
            "(vector 0 1 1 2 3 5 8 13 21 34 55 89)"
        );
    }

    #[test]
    fn test_vector_ref() {
        let code = r#"
        (def v (vector 1 2 3))
        (vector-ref v 1)
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "2");
    }

    #[test]
    fn test_vector_set() {
        let code = r#"
        (def v (vector 1 2 3))
        (vector-set! v 1 4)
        v
        "#;

        let mut interpreter = interpreter::Interpreter::with_stdlib();
        let result = interpreter.eval(code).unwrap();

        assert_eq!(result.borrow().to_string(), "(vector 1 4 3)");
    }
}

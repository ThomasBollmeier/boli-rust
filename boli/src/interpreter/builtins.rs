use super::values::*;
use std::rc::Rc;

pub struct Add {}

impl Add {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Add {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        calculate_value(|a, b| a.add(b), args, true)
    }
}

pub struct Sub {}

impl Sub {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Sub {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        calculate_value(|a, b| a.sub(b), args, true)
    }
}

pub struct Mul {}

impl Mul {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Mul {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        calculate_value(|a, b| a.mul(b), args, true)
    }
}

pub struct Div {}

impl Div {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Div {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        calculate_value(|a, b| a.div(b), args, true)
    }
}

pub struct Pow {}

impl Pow {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Pow {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        calculate_value(|a, b| a.pow(b), args, false)
    }
}

pub struct Rem {}

impl Rem {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Rem {
    fn call(&self, args: &Vec<Rc<dyn Value>>) -> EvalResult {
        calculate_value(|a, b| a.rem(b), args, true)
    }
}

fn calculate_value<F>(op: F, values: &Vec<Rc<dyn Value>>, left_associative: bool) -> EvalResult
where
    F: Fn(&Number, &Number) -> Number,
{
    let numbers = values_to_numbers(values)?;
    let calc_result = calculate(op, &numbers, left_associative);
    match calc_result {
        Number::Int(result) => Ok(Rc::new(IntValue { value: result })),
        Number::Float(result) => Ok(Rc::new(RealValue { value: result })),
    }
}

fn calculate<F>(op: F, numbers: &Vec<Number>, left_associative: bool) -> Number
where
    F: Fn(&Number, &Number) -> Number,
{
    if numbers.is_empty() {
        return Number::Int(0);
    }

    if numbers.len() == 1 {
        return numbers[0].clone();
    }

    if left_associative {
        let mut result = numbers[0].clone();
        for number in numbers.iter().skip(1) {
            result = op(&result, number);
        }
        result
    } else {
        let mut result = numbers[numbers.len() - 1].clone();
        for number in numbers.iter().rev().skip(1) {
            result = op(number, &result);
        }
        result
    }
}

fn values_to_numbers(vals: &Vec<Rc<dyn Value>>) -> Result<Vec<Number>, InterpreterError> {
    let mut numbers = vec![];

    for val in vals {
        let number = match val.get_type() {
            ValueType::Int => {
                let int_value = downcast_value::<IntValue>(val).unwrap();
                Number::Int(int_value.value)
            }
            ValueType::Real => {
                let real_value = downcast_value::<RealValue>(val).unwrap();
                Number::Float(real_value.value)
            }
            _ => return Err(InterpreterError::new(&format!("Invalid value: {:?}", val))),
        };
        numbers.push(number);
    }

    Ok(numbers)
}

#[derive(Clone)]
enum Number {
    Int(i64),
    Float(f64),
}

impl Number {
    fn add(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a + b),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 + b),
            (Number::Float(a), Number::Int(b)) => Number::Float(a + *b as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a + b),
        }
    }

    fn sub(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a - b),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 - b),
            (Number::Float(a), Number::Int(b)) => Number::Float(a - *b as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a - b),
        }
    }

    fn mul(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a * b),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 * b),
            (Number::Float(a), Number::Int(b)) => Number::Float(a * *b as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a * b),
        }
    }

    fn div(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a / b),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 / b),
            (Number::Float(a), Number::Int(b)) => Number::Float(a / *b as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a / b),
        }
    }

    fn pow(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a.pow(*b as u32)),
            (Number::Int(a), Number::Float(b)) => Number::Float((*a as f64).powf(*b)),
            (Number::Float(a), Number::Int(b)) => Number::Float(a.powi(*b as i32)),
            (Number::Float(a), Number::Float(b)) => Number::Float(a.powf(*b)),
        }
    }

    fn rem(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a % b),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 % b),
            (Number::Float(a), Number::Int(b)) => Number::Float(a % *b as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a % b),
        }
    }
}

use super::values::*;

pub struct Add {}

impl Add {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Add {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
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
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        calculate_value(|a, b| a.rem(b), args, true)
    }
}

pub struct Eq {}

impl Eq {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Eq {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        all_values(|a, b| a.eq(b), args)
    }
}

pub struct Gt {}

impl Gt {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Gt {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        all_values(|a, b| a.gt(b), args)
    }
}

pub struct Ge {}

impl Ge {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Ge {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        all_values(|a, b| a.ge(b), args)
    }
}

pub struct Lt {}

impl Lt {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Lt {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        all_values(|a, b| a.lt(b), args)
    }
}

pub struct Le {}

impl Le {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for Le {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        all_values(|a, b| a.le(b), args)
    }
}

fn calculate_value<F>(op: F, values: &Vec<ValueRef>, left_associative: bool) -> EvalResult
where
    F: Fn(&Number, &Number) -> Number,
{
    let numbers = values_to_numbers(values)?;
    let calc_result = calculate(op, &numbers, left_associative);
    match calc_result {
        Number::Int(result) => Ok(new_valueref(IntValue { value: result })),
        Number::Rational(n, d) => {
            let rational_value = RationalValue::new(n, d);
            if rational_value.denominator != 1 {
                Ok(new_valueref(rational_value))
            } else {
                Ok(new_valueref(IntValue {
                    value: rational_value.numerator,
                }))
            }
        }
        Number::Float(result) => Ok(new_valueref(RealValue { value: result })),
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

fn all_values<F>(op: F, values: &Vec<ValueRef>) -> EvalResult
where
    F: Fn(&Number, &Number) -> bool,
{
    let numbers = match values_to_numbers(values) {
        Ok(numbers) => numbers,
        Err(e) => return Err(e),
    };

    let op_result = all_numbers(op, &numbers);

    Ok(new_valueref(BoolValue { value: op_result }))
}

fn all_numbers<F>(op: F, numbers: &Vec<Number>) -> bool
where
    F: Fn(&Number, &Number) -> bool,
{
    if numbers.is_empty() {
        return false;
    }

    if numbers.len() == 1 {
        return true;
    }

    let mut iter = numbers.iter();
    let mut a = iter.next().unwrap();

    loop {
        let b = match iter.next() {
            Some(b) => b,
            None => break,
        };

        if !op(a, b) {
            return false;
        }

        a = b;
    }

    true
}

fn values_to_numbers(vals: &Vec<ValueRef>) -> Result<Vec<Number>, InterpreterError> {
    let mut numbers = vec![];

    for val in vals {
        let val = &borrow_value(val);
        let number = match val.get_type() {
            ValueType::Int => {
                let int_value = downcast_value::<IntValue>(val).unwrap();
                Number::Int(int_value.value)
            }
            ValueType::Rational => {
                let rational_value = downcast_value::<RationalValue>(val).unwrap();
                Number::Rational(rational_value.numerator, rational_value.denominator)
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
    Rational(i64, i64),
    Float(f64),
}

impl Number {
    fn to_float(&self) -> f64 {
        match self {
            Number::Int(a) => *a as f64,
            Number::Rational(n, d) => *n as f64 / *d as f64,
            Number::Float(a) => *a,
        }
    }

    fn add(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a + b),
            (Number::Int(a), Number::Rational(n, d)) => Number::Rational(a * d + n, *d),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 + b),
            (Number::Rational(n, d), Number::Int(b)) => Number::Rational(n + b * d, *d),
            (Number::Rational(n, d), Number::Rational(n2, d2)) => {
                Number::Rational(n * d2 + n2 * d, d * d2)
            }
            (Number::Rational(n, d), Number::Float(b)) => {
                Number::Float((*n as f64 / *d as f64) + b)
            }
            (Number::Float(a), Number::Int(b)) => Number::Float(a + *b as f64),
            (Number::Float(a), Number::Rational(n, d)) => Number::Float(a + *n as f64 / *d as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a + b),
        }
    }

    fn sub(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a - b),
            (Number::Int(a), Number::Rational(n, d)) => Number::Rational(a * d - n, *d),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 - b),
            (Number::Rational(n, d), Number::Int(b)) => Number::Rational(n - b * d, *d),
            (Number::Rational(n, d), Number::Rational(n2, d2)) => {
                Number::Rational(n * d2 - n2 * d, d * d2)
            }
            (Number::Rational(n, d), Number::Float(b)) => {
                Number::Float((*n as f64 / *d as f64) - b)
            }
            (Number::Float(a), Number::Int(b)) => Number::Float(a - *b as f64),
            (Number::Float(a), Number::Rational(n, d)) => Number::Float(a - *n as f64 / *d as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a - b),
        }
    }

    fn mul(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a * b),
            (Number::Int(a), Number::Rational(n, d)) => Number::Rational(a * n, *d),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 * b),
            (Number::Rational(n, d), Number::Int(b)) => Number::Rational(n * b, *d),
            (Number::Rational(n, d), Number::Rational(n2, d2)) => Number::Rational(n * n2, d * d2),
            (Number::Rational(n, d), Number::Float(b)) => {
                Number::Float((*n as f64 / *d as f64) * b)
            }
            (Number::Float(a), Number::Int(b)) => Number::Float(a * *b as f64),
            (Number::Float(a), Number::Rational(n, d)) => Number::Float(a * *n as f64 / *d as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a * b),
        }
    }

    fn div(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Rational(*a, *b),
            (Number::Int(a), Number::Rational(n, d)) => Number::Rational(a * d, *n),
            (Number::Int(a), Number::Float(b)) => Number::Float(*a as f64 / b),
            (Number::Rational(n, d), Number::Int(b)) => Number::Rational(*n, d * b),
            (Number::Rational(n, d), Number::Rational(n2, d2)) => Number::Rational(n * d2, d * n2),
            (Number::Rational(n, d), Number::Float(b)) => {
                Number::Float((*n as f64 / *d as f64) / b)
            }
            (Number::Float(a), Number::Int(b)) => Number::Float(a / *b as f64),
            (Number::Float(a), Number::Rational(n, d)) => Number::Float(a / *n as f64 / *d as f64),
            (Number::Float(a), Number::Float(b)) => Number::Float(a / b),
        }
    }

    fn pow(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a.pow(*b as u32)),
            (_, Number::Int(n)) => {
                let x = self.to_float();
                Number::Float(x.powi(*n as i32))
            }
            _ => {
                let x = self.to_float();
                let y = other.to_float();
                Number::Float(x.powf(y))
            }
        }
    }

    fn rem(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => Number::Int(a % b),
            _ => {
                let x = self.to_float();
                let y = other.to_float();
                Number::Float(x % y)
            }
        }
    }

    fn eq(&self, other: &Number) -> bool {
        match (self, other) {
            (Number::Int(a), Number::Int(b)) => a == b,
            (Number::Int(a), Number::Rational(n, d)) => *a * d == *n,
            (Number::Rational(n, d), Number::Int(b)) => *n == *b * d,
            (Number::Rational(n, d), Number::Rational(n2, d2)) => n * d2 == n2 * d,
            _ => {
                let a = self.to_float();
                let b = other.to_float();
                (a - b).abs() < f64::EPSILON
            }
        }
    }

    fn gt(&self, other: &Number) -> bool {
        let x = self.to_float();
        let y = other.to_float();
        x > y
    }

    fn ge(&self, other: &Number) -> bool {
        if self.eq(other) {
            return true;
        }
        self.gt(other)
    }

    fn lt(&self, other: &Number) -> bool {
        let x = self.to_float();
        let y = other.to_float();
        x < y
    }

    fn le(&self, other: &Number) -> bool {
        if self.eq(other) {
            return true;
        }
        self.lt(other)
    }
}

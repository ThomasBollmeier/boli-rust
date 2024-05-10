use core::str;

use super::values::*;

pub struct StrSub {}

impl StrSub {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StrSub {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let num_args = args.len();

        if num_args != 2 && num_args != 3 {
            return error("str-sub function expects two or three arguments");
        }

        let arg0 = borrow_value(&args[0]);
        let string = match arg0.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg0).unwrap(),
            _ => return error("str-sub function expects a string as the first argument"),
        };

        let arg1 = borrow_value(&args[1]);
        let start = match arg1.get_type() {
            ValueType::Int => downcast_value::<IntValue>(&arg1).unwrap().value as usize,
            _ => return error("str-sub function expects an integer as the second argument"),
        };

        if num_args == 2 {
            Ok(new_valueref(StrValue {
                value: string.value[start..].to_string(),
            }))
        } else {
            let arg2 = borrow_value(&args[2]);
            let length = match arg2.get_type() {
                ValueType::Int => downcast_value::<IntValue>(&arg2).unwrap().value as usize,
                _ => return error("str-sub function expects an integer as the third argument"),
            };
            let end = start + length;

            Ok(new_valueref(StrValue {
                value: string.value[start..end].to_string(),
            }))
        }
    }
}

pub struct StrReplace {}

impl StrReplace {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StrReplace {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 3 {
            return error("str-replace function expects exactly three arguments");
        }

        let arg0 = borrow_value(&args[0]);
        let string = match arg0.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg0).unwrap(),
            _ => return error("str-replace function expects a string as the first argument"),
        };

        let arg1 = borrow_value(&args[1]);
        let pattern = match arg1.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg1).unwrap(),
            _ => return error("str-replace function expects a string as the second argument"),
        };

        let arg2 = borrow_value(&args[2]);
        let replacement = match arg2.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg2).unwrap(),
            _ => return error("str-replace function expects a string as the third argument"),
        };

        Ok(new_valueref(StrValue {
            value: string.value.replace(&pattern.value, &replacement.value),
        }))
    }
}

pub struct StrConcat {}

impl StrConcat {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StrConcat {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let mut result = String::new();

        for arg in args {
            let value = borrow_value(&arg);
            match value.get_type() {
                ValueType::Str => {
                    let string = downcast_value::<StrValue>(&value).unwrap();
                    result.push_str(&string.value);
                }
                _ => return error("str-concat function expects only strings as arguments"),
            }
        }

        Ok(new_valueref(StrValue { value: result }))
    }
}

pub struct StrUpper {}

impl StrUpper {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StrUpper {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("str-upper function expects exactly one argument");
        }

        let arg0 = borrow_value(&args[0]);
        let string = match arg0.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg0).unwrap(),
            _ => return error("str-upper function expects a string as the first argument"),
        };

        Ok(new_valueref(StrValue {
            value: string.value.to_uppercase(),
        }))
    }
}

pub struct StrLower {}

impl StrLower {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StrLower {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("str-lower function expects exactly one argument");
        }

        let arg0 = borrow_value(&args[0]);
        let string = match arg0.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg0).unwrap(),
            _ => return error("str-lower function expects a string as the first argument"),
        };

        Ok(new_valueref(StrValue {
            value: string.value.to_lowercase(),
        }))
    }
}

pub struct StrToInt {}

impl StrToInt {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StrToInt {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("str-to-int function expects exactly one argument");
        }

        let arg0 = borrow_value(&args[0]);
        let string = match arg0.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg0).unwrap(),
            _ => return error("str-to-int function expects a string as the first argument"),
        };

        let string_val = str::replace(&string.value, ".", "");

        match string_val.parse::<i64>() {
            Ok(value) => Ok(new_valueref(IntValue { value })),
            Err(_) => Ok(new_valueref(BoolValue { value: false })),
        }
    }
}

pub struct StrToReal {}

impl StrToReal {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StrToReal {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("str-to-real function expects exactly one argument");
        }

        let arg0 = borrow_value(&args[0]);
        let string = match arg0.get_type() {
            ValueType::Str => downcast_value::<StrValue>(&arg0).unwrap(),
            _ => return error("str-to-real function expects a string as the first argument"),
        };

        let mut string_val = str::replace(&string.value, ".", "");
        string_val = string_val.replace(",", ".");

        match string_val.parse::<f64>() {
            Ok(value) => Ok(new_valueref(RealValue { value })),
            Err(_) => Ok(new_valueref(BoolValue { value: false })),
        }
    }
}

use core::str;
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::rc::Rc;

use crate::frontend::lexer::tokens::Token;

use super::environment::{Environment, EnvironmentBuilder};
use super::{AstRef, Interpreter};

pub mod stream;

#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub enum ValueType {
    Nil,
    Bool,
    Int,
    Real,
    Str,
    Symbol,
    Quote,
    Pair,
    Vector,
    Stream,
    StructType,
    Struct,
    Lambda,
    BuiltInFunction,
    TailCall,
    Spread,
}

pub trait Value: Display + Debug {
    fn get_type(&self) -> ValueType;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

pub type ValueRef = Rc<RefCell<dyn Value>>;

pub fn new_valueref<T: Value + 'static>(value: T) -> ValueRef {
    Rc::new(RefCell::new(value))
}

pub fn borrow_value<'a>(value: &'a ValueRef) -> Ref<'a, dyn Value> {
    value.borrow()
}

pub fn borrow_mut_value<'a>(value: &'a ValueRef) -> RefMut<'a, dyn Value> {
    value.borrow_mut()
}

pub fn downcast_value<'a, T: 'static>(value: &'a Ref<dyn Value>) -> Option<&'a T> {
    value.as_any().downcast_ref::<T>()
}

#[derive(Debug)]
pub struct NilValue {}

impl Value for NilValue {
    fn get_type(&self) -> ValueType {
        ValueType::Nil
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for NilValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "nil")
    }
}

#[derive(Debug)]
pub struct BoolValue {
    pub value: bool,
}

impl Value for BoolValue {
    fn get_type(&self) -> ValueType {
        ValueType::Bool
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for BoolValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.value { "#true" } else { "#false" })
    }
}

impl ComparableEq for BoolValue {
    fn is_equal(&self, other: &ValueRef) -> bool {
        if let Some(other) = downcast_value::<BoolValue>(&other.borrow()) {
            self.value == other.value
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct IntValue {
    pub value: i64,
}

impl Value for IntValue {
    fn get_type(&self) -> ValueType {
        ValueType::Int
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for IntValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl ComparableEq for IntValue {
    fn is_equal(&self, other: &ValueRef) -> bool {
        if let Some(other) = downcast_value::<IntValue>(&other.borrow()) {
            self.value == other.value
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct RealValue {
    pub value: f64,
}

impl Value for RealValue {
    fn get_type(&self) -> ValueType {
        ValueType::Real
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for RealValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value_str = format!("{:?}", self.value).replace(".", ",");
        write!(f, "{value_str}")
    }
}

#[derive(Debug)]
pub struct StrValue {
    pub value: String,
}

impl Value for StrValue {
    fn get_type(&self) -> ValueType {
        ValueType::Str
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for StrValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

impl Countable for StrValue {
    fn count(&self) -> usize {
        self.value.chars().count()
    }
}

impl ComparableEq for StrValue {
    fn is_equal(&self, other: &ValueRef) -> bool {
        if let Some(other) = downcast_value::<StrValue>(&other.borrow()) {
            self.value == other.value
        } else {
            false
        }
    }
}

#[derive(Debug)]
pub struct SymbolValue {
    pub value: String,
}

impl SymbolValue {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }
}

impl Value for SymbolValue {
    fn get_type(&self) -> ValueType {
        ValueType::Symbol
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for SymbolValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}", self.value)
    }
}

pub struct QuoteValue {
    pub token: Token,
}

impl QuoteValue {
    pub fn new(token: &Token) -> Self {
        Self {
            token: token.clone(),
        }
    }
}

impl Value for QuoteValue {
    fn get_type(&self) -> ValueType {
        ValueType::Quote
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for QuoteValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token_str = self.token.get_display_str().unwrap_or("".to_string());
        write!(f, "'{}", token_str)
    }
}

impl Debug for QuoteValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token_str = self.token.get_display_str().unwrap_or("".to_string());
        write!(f, "'{}", token_str)
    }
}

#[derive(Debug, Clone)]
pub struct PairValue {
    pub left: ValueRef,
    pub right: ValueRef,
}

impl PairValue {
    pub fn new(left: &ValueRef, right: &ValueRef) -> Self {
        Self {
            left: left.clone(),
            right: right.clone(),
        }
    }

    pub fn is_list(&self) -> bool {
        let mut current = new_valueref(self.clone());
        loop {
            let current_type = current.borrow().get_type();

            match current_type {
                ValueType::Nil => return true,
                ValueType::Pair => {
                    current = Self::get_right(&current);
                }
                _ => return false,
            }
        }
    }

    pub fn count(&self) -> usize {
        let mut count = 0;
        let mut current = new_valueref(self.clone());

        loop {
            let current_type = current.borrow().get_type();

            match current_type {
                ValueType::Nil => return count,
                ValueType::Pair => {
                    count += 1;
                    current = Self::get_right(&current);
                }
                _ => return count,
            }
        }
    }

    pub fn concat(&self, other: &ValueRef) -> ValueRef {
        let mut elements = self.get_elements();
        let other = borrow_value(other);
        let other = downcast_value::<PairValue>(&other).unwrap();
        let other_elements = other.get_elements();
        elements.extend(other_elements);

        let mut result = new_valueref(NilValue {});
        for element in elements.iter().rev() {
            result = new_valueref(PairValue::new(element, &result));
        }

        result
    }

    fn get_elements(&self) -> Vec<ValueRef> {
        let mut elements = vec![];
        let mut current = new_valueref(self.clone());

        loop {
            let current_type = current.borrow().get_type();

            match current_type {
                ValueType::Nil => return elements,
                ValueType::Pair => {
                    {
                        let pair = borrow_value(&current);
                        let pair = downcast_value::<PairValue>(&pair).unwrap();
                        elements.push(pair.left.clone());
                    }
                    current = Self::get_right(&current);
                }
                _ => return elements,
            }
        }
    }

    fn get_right(value: &ValueRef) -> ValueRef {
        let pair = borrow_value(value);
        let pair = downcast_value::<PairValue>(&pair).unwrap();
        pair.right.clone()
    }
}

impl Value for PairValue {
    fn get_type(&self) -> ValueType {
        ValueType::Pair
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for PairValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.is_list() {
            write!(f, "({} . {})", self.left.borrow(), self.right.borrow())
        } else {
            let elements = self.get_elements();
            let elements_str: Vec<String> =
                elements.iter().map(|e| format!("{}", e.borrow())).collect();
            write!(f, "(list {})", elements_str.join(" "))
        }
    }
}

#[derive(Debug)]
pub struct VectorValue {
    pub elements: Vec<ValueRef>,
}

impl Value for VectorValue {
    fn get_type(&self) -> ValueType {
        ValueType::Vector
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for VectorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let elements_str: Vec<String> = self
            .elements
            .iter()
            .map(|e| format!("{}", e.borrow()))
            .collect();
        write!(f, "(vector {})", elements_str.join(" "))
    }
}

impl Countable for VectorValue {
    fn count(&self) -> usize {
        self.elements.len()
    }
}

#[derive(Debug)]
pub struct StructTypeValue {
    pub name: String,
    pub fields: Vec<String>,
}

impl StructTypeValue {
    pub fn new(name: &str, fields: &Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            fields: fields.clone(),
        }
    }
}

impl Value for StructTypeValue {
    fn get_type(&self) -> ValueType {
        ValueType::StructType
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for StructTypeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fields_str: Vec<String> = self.fields.iter().map(|f| f.to_string()).collect();
        write!(f, "(def-struct {} {})", self.name, fields_str.join(" "))
    }
}

pub struct CreateStructValue {
    struct_type: ValueRef,
}

impl CreateStructValue {
    pub fn new(struct_type: &ValueRef) -> Self {
        Self {
            struct_type: struct_type.clone(),
        }
    }
}

impl Callable for CreateStructValue {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let struct_type = borrow_value(&self.struct_type);
        let struct_type = downcast_value::<StructTypeValue>(&struct_type);
        if struct_type.is_none() {
            return error("Invalid struct type");
        }
        let struct_type = struct_type.unwrap();

        if args.len() != struct_type.fields.len() {
            return error("Number of arguments differs from number of fields");
        }

        let mut values = HashMap::new();
        for (i, field) in struct_type.fields.iter().enumerate() {
            let entry = StructEntry {
                key: new_valueref(StrValue {
                    value: field.clone(),
                }),
                value: args[i].clone(),
            };
            values.insert(field.clone(), entry);
        }

        Ok(new_valueref(StructValue::new(&self.struct_type, values)))
    }
}

pub struct IsStructType {
    struct_type: ValueRef,
}

impl IsStructType {
    pub fn new(struct_type: &ValueRef) -> Self {
        Self {
            struct_type: struct_type.clone(),
        }
    }
}

impl Callable for IsStructType {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("function expects exactly one argument");
        }

        let struct_value = borrow_value(&args[0]);
        let struct_value = downcast_value::<StructValue>(&struct_value);
        if struct_value.is_none() {
            return Ok(new_valueref(BoolValue { value: false }));
        }
        let struct_value = struct_value.unwrap();

        let struct_type = borrow_value(&self.struct_type);
        let struct_type = downcast_value::<StructTypeValue>(&struct_type);
        if struct_type.is_none() {
            return error("Invalid struct type");
        }
        let struct_type = struct_type.unwrap();

        if let Some(arg_type) = &struct_value.struct_type {
            let arg_type = borrow_value(&arg_type);
            let arg_type = downcast_value::<StructTypeValue>(&arg_type);
            match arg_type {
                Some(arg_type) => Ok(new_valueref(BoolValue {
                    value: arg_type.name == struct_type.name,
                })),
                None => Ok(new_valueref(BoolValue { value: false })),
            }
        } else {
            Ok(new_valueref(BoolValue { value: false }))
        }
    }
}

pub struct GetStructField {
    field: String,
}

impl GetStructField {
    pub fn new(field: &str) -> Self {
        Self {
            field: field.to_string(),
        }
    }
}

impl Callable for GetStructField {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("getter function expects exactly one argument");
        }

        let struct_value = borrow_value(&args[0]);
        let struct_value = downcast_value::<StructValue>(&struct_value);
        if struct_value.is_none() {
            return error("getter function expects a struct");
        }
        let struct_value = struct_value.unwrap();

        if let Some(entry) = struct_value.values.get(&self.field) {
            Ok(entry.value.clone())
        } else {
            error("Field not found")
        }
    }
}

pub struct SetStructField {
    field: String,
}

impl SetStructField {
    pub fn new(field: &str) -> Self {
        Self {
            field: field.to_string(),
        }
    }
}

impl Callable for SetStructField {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("set-struct function expects exactly two arguments");
        }

        let mut struct_value = borrow_mut_value(&args[0]);
        let struct_value = struct_value.as_any_mut().downcast_mut::<StructValue>();
        if struct_value.is_none() {
            return error("set-struct function expects a struct");
        }
        let struct_value = struct_value.unwrap();

        let new_entry = StructEntry {
            key: new_valueref(StrValue {
                value: self.field.clone(),
            }),
            value: args[1].clone(),
        };
        struct_value.values.insert(self.field.clone(), new_entry);

        Ok(new_valueref(NilValue {}))
    }
}

#[derive(Debug)]
pub struct StructEntry {
    pub key: ValueRef,
    pub value: ValueRef,
}

#[derive(Debug)]
pub struct StructValue {
    pub struct_type: Option<ValueRef>,
    pub values: HashMap<String, StructEntry>,
    is_set: bool,
}

impl StructValue {
    pub fn new(struct_type: &ValueRef, values: HashMap<String, StructEntry>) -> Self {
        Self {
            struct_type: Some(struct_type.clone()),
            values,
            is_set: false,
        }
    }

    pub fn new_hash_table() -> Self {
        Self {
            struct_type: None,
            values: HashMap::new(),
            is_set: false,
        }
    }

    pub fn new_set() -> Self {
        Self {
            struct_type: None,
            values: HashMap::new(),
            is_set: true,
        }
    }
}

impl Value for StructValue {
    fn get_type(&self) -> ValueType {
        ValueType::Struct
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for StructValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.struct_type {
            Some(struct_type) => {
                let struct_type = borrow_value(struct_type);
                let struct_type = downcast_value::<StructTypeValue>(&struct_type).unwrap();
                let values_str = &struct_type
                    .fields
                    .iter()
                    .filter_map(|field| {
                        self.values
                            .get(field)
                            .map(|entry| format!("'{} {}", field, entry.value.borrow()))
                    })
                    .collect::<Vec<String>>();

                write!(f, "(struct {} {})", struct_type.name, values_str.join(" "))
            }
            None => {
                if !self.is_set {
                    let mut keys = self
                        .values
                        .keys()
                        .map(|k| k.clone())
                        .collect::<Vec<String>>();
                    keys.sort();

                    let values_str = keys
                        .iter()
                        .map(|key| {
                            format!("'{} {}", key, self.values.get(key).unwrap().value.borrow())
                        })
                        .collect::<Vec<String>>();

                    write!(f, "(hash-table {})", values_str.join(" "))
                } else {
                    let values_str = self
                        .values
                        .values()
                        .map(|entry| format!("{}", entry.key.borrow()))
                        .collect::<Vec<String>>();

                    write!(f, "(set {})", values_str.join(" "))
                }
            }
        }
    }
}

pub type EvalResult = Result<ValueRef, InterpreterError>;

pub fn error(message: &str) -> EvalResult {
    Err(InterpreterError::new(message))
}

pub trait Callable {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult;
}

pub trait Countable {
    fn count(&self) -> usize;
}

pub trait ComparableEq {
    fn is_equal(&self, other: &ValueRef) -> bool;
}

pub enum LambdaVariant {
    Arities(HashMap<usize, (Vec<String>, AstRef)>),
    Variadic(Vec<String>, String, AstRef),
}

impl LambdaVariant {
    pub fn new(parameters: &Vec<String>, variadic: &Option<String>, body: &AstRef) -> Self {
        match variadic {
            Some(variadic) => {
                LambdaVariant::Variadic(parameters.clone(), variadic.clone(), body.clone())
            }
            None => {
                let mut arities = HashMap::new();
                arities.insert(parameters.len(), (parameters.clone(), body.clone()));
                LambdaVariant::Arities(arities)
            }
        }
    }
}

pub struct LambdaValue {
    pub name: Option<String>,
    pub variant: LambdaVariant,
    pub env: Rc<RefCell<Environment>>,
}

impl LambdaValue {
    pub fn new(
        name: Option<String>,
        parameters: Vec<String>,
        variadic: Option<String>,
        body: &AstRef,
        env: &Rc<RefCell<Environment>>,
    ) -> Self {
        Self {
            name: name.clone(),
            variant: LambdaVariant::new(&parameters, &variadic, body),
            env: env.clone(),
        }
    }

    pub fn merge_lambda(&mut self, other: &LambdaValue) -> Result<(), InterpreterError> {
        match &mut self.variant {
            LambdaVariant::Arities(arities) => match &other.variant {
                LambdaVariant::Arities(other_arities) => {
                    for (arity, (parameters, body)) in other_arities {
                        arities.insert(*arity, (parameters.clone(), body.clone()));
                    }
                    Ok(())
                }
                LambdaVariant::Variadic(_, _, _) => {
                    Err(InterpreterError::new("Arities already defined"))
                }
            },
            LambdaVariant::Variadic(_, _, _) => Err(InterpreterError::new(
                "Cannot have multiple arities for variadic lambda",
            )),
        }
    }

    fn init_call_env(
        &self,
        args: &Vec<ValueRef>,
    ) -> Result<Rc<RefCell<Environment>>, InterpreterError> {
        match &self.variant {
            LambdaVariant::Arities(arities) => {
                let arity_variant = arities.get(&args.len());
                match arity_variant {
                    Some((parameters, _)) => self.init_call_env_with_params(parameters, None, args),
                    None => Err(InterpreterError::new(
                        "Number of arguments differs from number of parameters",
                    )),
                }
            }
            LambdaVariant::Variadic(parameters, variadic, _) => {
                self.init_call_env_with_params(parameters, Some(variadic.clone()), args)
            }
        }
    }

    fn init_call_env_with_params(
        &self,
        parameters: &Vec<String>,
        variadic: Option<String>,
        args: &Vec<ValueRef>,
    ) -> Result<Rc<RefCell<Environment>>, InterpreterError> {
        let num_args = args.len();
        let num_params = parameters.len();

        if variadic.is_some() && num_args < num_params {
            return Err(InterpreterError::new(
                "Number of arguments differs from number of parameters",
            ));
        }

        let call_env = EnvironmentBuilder::new().parent(&self.env.clone()).build();

        for (i, param) in parameters.iter().enumerate() {
            call_env
                .borrow_mut()
                .set(param.to_string(), args[i].clone());
        }

        if let Some(var_param) = &variadic {
            let arg_list = if num_args > num_params {
                let elements = args
                    .iter()
                    .skip(num_params)
                    .map(|val| val.clone())
                    .collect();
                VectorValue { elements }
            } else {
                VectorValue { elements: vec![] }
            };
            call_env
                .borrow_mut()
                .set(var_param.to_string(), new_valueref(arg_list));
        }

        Ok(call_env)
    }

    fn get_body(&self, num_args: usize) -> Result<AstRef, InterpreterError> {
        match &self.variant {
            LambdaVariant::Arities(arities) => {
                let arity_variant = arities.get(&num_args);
                match arity_variant {
                    Some((_, body)) => Ok(body.clone()),
                    None => Err(InterpreterError::new(
                        "Number of arguments differs from number of parameters",
                    )),
                }
            }
            LambdaVariant::Variadic(_, _, body) => Ok(body.clone()),
        }
    }
}

impl Value for LambdaValue {
    fn get_type(&self) -> ValueType {
        ValueType::Lambda
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for LambdaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "<lambda {}>", name)
        } else {
            write!(f, "<lambda>")
        }
    }
}

impl Debug for LambdaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(name) = &self.name {
            write!(f, "<lambda {}>", name)
        } else {
            write!(f, "<lambda>")
        }
    }
}

impl Callable for LambdaValue {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let call_env = self.init_call_env(args)?;
        let mut interpreter = Interpreter::with_environment(&call_env);

        let body = self.get_body(args.len())?;
        body.borrow().accept(&mut interpreter);
        interpreter.stack.pop().unwrap()
    }
}

pub struct BuiltInFunctionValue {
    pub name: String,
    pub function: Rc<dyn Callable>,
}

impl BuiltInFunctionValue {
    pub fn new(name: &str, function: &Rc<dyn Callable>) -> Self {
        Self {
            name: name.to_string(),
            function: function.clone(),
        }
    }
}

impl Display for BuiltInFunctionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl Debug for BuiltInFunctionValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<built-in function {}>", self.name)
    }
}

impl Value for BuiltInFunctionValue {
    fn get_type(&self) -> ValueType {
        ValueType::BuiltInFunction
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Callable for BuiltInFunctionValue {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        self.function.call(args)
    }
}

pub struct TailCallValue {
    pub arguments: Vec<ValueRef>,
}

impl Value for TailCallValue {
    fn get_type(&self) -> ValueType {
        ValueType::TailCall
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for TailCallValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<tail call>")
    }
}

impl Debug for TailCallValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<tail call>")
    }
}

pub struct SpreadValue {
    pub elements: Vec<ValueRef>,
}

impl Value for SpreadValue {
    fn get_type(&self) -> ValueType {
        ValueType::Spread
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Display for SpreadValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<spread>")
    }
}

impl Debug for SpreadValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<spread>")
    }
}

#[derive(Debug)]
pub struct InterpreterError {
    pub message: String,
}

impl InterpreterError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InterpreterError {
    fn description(&self) -> &str {
        &self.message
    }
}

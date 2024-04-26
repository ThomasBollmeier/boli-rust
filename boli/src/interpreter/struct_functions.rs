use crate::frontend::lexer::tokens::TokenType;

use super::values::*;

pub struct StructGet {}

impl StructGet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StructGet {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("struct-get expects 2 arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<StructValue>(arg0);
        if arg0.is_none() {
            return error("struct-get expects a struct as the first argument");
        }
        let struct_value = arg0.unwrap();

        let arg1 = &borrow_value(&args[1]);
        let arg1 = downcast_value::<ListValue>(arg1);
        if arg1.is_none() {
            return error("struct-get expects a list as the second argument");
        }
        let path = arg1.unwrap();

        let field = path.elements[0].clone();
        let field = &borrow_value(&field);
        let field = downcast_value::<QuoteValue>(field);
        if field.is_none() {
            return error("struct-get expects a list of quoted identifiers as the second argument");
        }

        let quoted_field = field.unwrap();
        let field_token = quoted_field.token.clone();

        if field_token.token_type != TokenType::Identifier {
            return error("struct-get expects a list of quoted identifiers as the second argument");
        }

        let field_name = field_token.get_string_value().unwrap();
        let field_value = struct_value.values.get(&field_name);

        if field_value.is_none() {
            return error(&format!("field '{}' not found in struct", field_name));
        }

        let field_value = field_value.unwrap();

        if path.elements.len() == 1 {
            return Ok(field_value.clone());
        }

        let new_path = new_valueref(ListValue {
            elements: path.elements[1..].to_vec(),
        });

        self.call(&vec![field_value.clone(), new_path])
    }
}

pub struct StructSet {}

impl StructSet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for StructSet {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 3 {
            return error("struct-set expects 3 arguments");
        }

        let mut arg0 = borrow_mut_value(&args[0]);
        let arg0 = arg0.as_any_mut().downcast_mut::<StructValue>();
        if arg0.is_none() {
            return error("struct-set expects a struct as the first argument");
        }
        let struct_value = arg0.unwrap();

        let arg1 = &borrow_value(&args[1]);
        let arg1 = downcast_value::<ListValue>(arg1);
        if arg1.is_none() {
            return error("struct-set expects a list as the second argument");
        }
        let path = arg1.unwrap();

        let field = path.elements[0].clone();
        let field = &borrow_value(&field);
        let field = downcast_value::<QuoteValue>(field);
        if field.is_none() {
            return error("struct-set expects a list of quoted identifiers as the second argument");
        }

        let quoted_field = field.unwrap();
        let field_token = quoted_field.token.clone();

        if field_token.token_type != TokenType::Identifier {
            return error("struct-set expects a list of quoted identifiers as the second argument");
        }

        let field_name = field_token.get_string_value().unwrap();

        let new_value = args[2].clone();

        if path.elements.len() == 1 {
            struct_value.values.insert(field_name, new_value);
            return Ok(new_valueref(NilValue {}));
        }

        let field_value = struct_value.values.get(&field_name);
        if field_value.is_none() {
            return error(&format!("field '{}' not found in struct", field_name));
        }
        let inner_struct = field_value.unwrap().clone();

        let new_path = new_valueref(ListValue {
            elements: path.elements[1..].to_vec(),
        });

        self.call(&vec![inner_struct, new_path, new_value])
    }
}

pub struct CreateHashTable {}

impl CreateHashTable {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for CreateHashTable {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 0 {
            return error("create-hash-table expects 0 arguments");
        }

        Ok(new_valueref(StructValue::new_hash_table()))
    }
}

use super::values::*;
use crate::frontend::lexer::tokens::TokenType;

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
        let arg1 = downcast_value::<VectorValue>(arg1);
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
        let entry = struct_value.values.get(&field_name);

        if entry.is_none() {
            return error(&format!("field '{}' not found in struct", field_name));
        }

        let entry = entry.unwrap();

        if path.elements.len() == 1 {
            return Ok(entry.value.clone());
        }

        let new_path = new_valueref(VectorValue {
            elements: path.elements[1..].to_vec(),
        });

        self.call(&vec![entry.value.clone(), new_path])
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
        let arg1 = downcast_value::<VectorValue>(arg1);
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

        let new_entry = StructEntry {
            key: new_valueref(SymbolValue {
                value: field_name.clone(),
            }),
            value: args[2].clone(),
        };

        if path.elements.len() == 1 {
            struct_value.values.insert(field_name, new_entry);
            return Ok(new_valueref(NilValue {}));
        }

        let field_value = struct_value.values.get(&field_name);
        if field_value.is_none() {
            return error(&format!("field '{}' not found in struct", field_name));
        }
        let inner_struct = field_value.unwrap().value.clone();

        let new_path = new_valueref(VectorValue {
            elements: path.elements[1..].to_vec(),
        });

        self.call(&vec![inner_struct, new_path, new_entry.value])
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

pub struct HashLength {}

impl HashLength {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for HashLength {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("hash-length expects 1 argument");
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<StructValue>(arg0);
        if arg0.is_none() {
            return error("hash-length expects a hash table as the first argument");
        }
        let hash_table = arg0.unwrap();
        if hash_table.struct_type.is_some() {
            return error("hash-length expects a hash table as the first argument");
        }

        Ok(new_valueref(IntValue {
            value: hash_table.values.len() as i64,
        }))
    }
}

pub struct HashKeys {}

impl HashKeys {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for HashKeys {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("hash-keys expects 1 argument");
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<StructValue>(arg0);
        if arg0.is_none() {
            return error("hash-keys expects a hash table as the first argument");
        }
        let hash_table = arg0.unwrap();
        if hash_table.struct_type.is_some() {
            return error("hash-keys expects a hash table as the first argument");
        }

        let keys = hash_table
            .values
            .values()
            .map(|entry| entry.key.clone())
            .collect();

        Ok(new_valueref(VectorValue { elements: keys }))
    }
}

fn get_key(value: &ValueRef) -> String {
    let value = borrow_value(value);

    format!("{:?}", value.to_string())
}

pub struct HashContains {}

impl HashContains {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for HashContains {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("hash-contains? expects 2 arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<StructValue>(arg0);
        if arg0.is_none() {
            return error("hash-contains? expects a hash table as the first argument");
        }
        let hash_table = arg0.unwrap();
        if hash_table.struct_type.is_some() {
            return error("hash-contains? expects a hash table as the first argument");
        }

        let key = get_key(&args[1]);

        let value = hash_table.values.get(&key);

        Ok(new_valueref(BoolValue {
            value: value.is_some(),
        }))
    }
}

pub struct HashGet {}

impl HashGet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for HashGet {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("hash-get expects 2 arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<StructValue>(arg0);
        if arg0.is_none() {
            return error("hash-get expects a hash table as the first argument");
        }
        let hash_table = arg0.unwrap();
        if hash_table.struct_type.is_some() {
            return error("hash-get expects a hash table as the first argument");
        }

        let key = get_key(&args[1]);

        let entry = hash_table.values.get(&key);

        if entry.is_none() {
            return error(&format!("key '{}' not found in hash table", key));
        }

        Ok(entry.unwrap().value.clone())
    }
}

pub struct HashSetBang {}

impl HashSetBang {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for HashSetBang {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 3 {
            return error("hash-set! expects 3 arguments");
        }

        let mut arg0 = borrow_mut_value(&args[0]);
        let arg0 = arg0.as_any_mut().downcast_mut::<StructValue>();
        if arg0.is_none() {
            return error("hash-set! expects a hash table as the first argument");
        }
        let hash_table = arg0.unwrap();
        if hash_table.struct_type.is_some() {
            return error("hash-set! expects a hash table as the first argument");
        }

        let key = get_key(&args[1]);

        let new_entry = StructEntry {
            key: args[1].clone(),
            value: args[2].clone(),
        };

        hash_table.values.insert(key, new_entry);

        Ok(new_valueref(NilValue {}))
    }
}

pub struct HashRemoveBang {}

impl HashRemoveBang {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for HashRemoveBang {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("hash-remove! expects 2 arguments");
        }

        let mut arg0 = borrow_mut_value(&args[0]);
        let arg0 = arg0.as_any_mut().downcast_mut::<StructValue>();
        if arg0.is_none() {
            return error("hash-remove! expects a hash table as the first argument");
        }
        let hash_table = arg0.unwrap();
        if hash_table.struct_type.is_some() {
            return error("hash-remove! expects a hash table as the first argument");
        }

        let key = get_key(&args[1]);

        hash_table.values.remove(&key);

        Ok(new_valueref(NilValue {}))
    }
}

pub struct CreateSet {}

impl CreateSet {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for CreateSet {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        let mut ret = StructValue::new_set();
        for arg in args {
            let key = get_key(arg);
            let entry = StructEntry {
                key: arg.clone(),
                value: new_valueref(NilValue {}),
            };
            ret.values.insert(key, entry);
        }

        Ok(new_valueref(ret))
    }
}

pub struct SetLength {}

impl SetLength {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for SetLength {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 1 {
            return error("set-length expects 1 argument");
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<StructValue>(arg0);
        if arg0.is_none() {
            return error("set-length expects a set as the first argument");
        }
        let set = arg0.unwrap();
        if set.struct_type.is_some() {
            return error("set-length expects a set as the first argument");
        }

        Ok(new_valueref(IntValue {
            value: set.values.len() as i64,
        }))
    }
}

pub struct SetContains {}

impl SetContains {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for SetContains {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("set-contains? expects 2 arguments");
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<StructValue>(arg0);
        if arg0.is_none() {
            return error("set-contains? expects a set as the first argument");
        }
        let set = arg0.unwrap();
        if set.struct_type.is_some() {
            return error("set-contains? expects a set as the first argument");
        }

        let key = get_key(&args[1]);

        let value = set.values.get(&key);

        Ok(new_valueref(BoolValue {
            value: value.is_some(),
        }))
    }
}

pub struct SetAddBang {}

impl SetAddBang {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for SetAddBang {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("set-add! expects 2 arguments");
        }

        let mut arg0 = borrow_mut_value(&args[0]);
        let arg0 = arg0.as_any_mut().downcast_mut::<StructValue>();
        if arg0.is_none() {
            return error("set-add! expects a set as the first argument");
        }
        let set = arg0.unwrap();
        if set.struct_type.is_some() {
            return error("set-add! expects a set as the first argument");
        }

        let key = get_key(&args[1]);
        let entry = StructEntry {
            key: args[1].clone(),
            value: new_valueref(NilValue {}),
        };

        set.values.insert(key, entry);

        Ok(new_valueref(NilValue {}))
    }
}

pub struct SetRemoveBang {}

impl SetRemoveBang {
    pub fn new() -> Self {
        Self {}
    }
}

impl Callable for SetRemoveBang {
    fn call(&self, args: &Vec<ValueRef>) -> EvalResult {
        if args.len() != 2 {
            return error("set-remove! expects 2 arguments");
        }

        let mut arg0 = borrow_mut_value(&args[0]);
        let arg0 = arg0.as_any_mut().downcast_mut::<StructValue>();
        if arg0.is_none() {
            return error("set-remove! expects a set as the first argument");
        }
        let set = arg0.unwrap();
        if set.struct_type.is_some() {
            return error("set-remove! expects a set as the first argument");
        }

        let key = get_key(&args[1]);

        set.values.remove(&key);

        Ok(new_valueref(NilValue {}))
    }
}

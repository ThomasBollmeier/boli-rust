use super::{
    environment::EnvironmentRef, module_mgmt::ModuleDirRef, Interpreter, InterpreterError, ValueRef,
};
use crate::interpreter::module_mgmt::extension::new_extension_dir;
use std::collections::HashMap;

mod list;
mod seq_collection;
mod stream;
mod string;
mod vector;

pub fn create_stdlib() -> ModuleDirRef {
    let stdlib = new_extension_dir("");

    let vector_ext = vector::create_vector_extension();
    stdlib.borrow_mut().add_extension(&vector_ext);

    let list_ext = list::create_list_extension(&vector_ext);
    stdlib.borrow_mut().add_extension(&list_ext);

    let stream_ext = stream::create_stream_extension();
    stdlib.borrow_mut().add_extension(&stream_ext);

    let string_ext = string::create_string_extension();
    stdlib.borrow_mut().add_extension(&string_ext);

    let seq_collection =
        seq_collection::create_seq_collection_extension(&vec![vector_ext, list_ext, string_ext]);
    stdlib.borrow_mut().add_extension(&seq_collection);

    stdlib
}

fn load_module_code(
    env: &EnvironmentRef,
    code: &str,
) -> Result<HashMap<String, ValueRef>, InterpreterError> {
    let mut interpreter = Interpreter::with_environment(env);
    interpreter.eval(code)?;
    Ok(env.borrow().get_exported_values())
}

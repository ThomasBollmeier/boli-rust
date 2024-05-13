use super::{
    environment::EnvironmentRef, module_mgmt::ModuleDirRef, Interpreter, InterpreterError, ValueRef,
};
use crate::interpreter::module_mgmt::extension::new_extension_dir;
use std::collections::HashMap;

mod list;
mod vector;

pub fn create_stdlib() -> ModuleDirRef {
    let stdlib = new_extension_dir("");

    let vector_ext = vector::create_vector_extension();

    stdlib.borrow_mut().add_extension(&vector_ext);

    stdlib
        .borrow_mut()
        .add_extension(&list::create_list_extension(&vector_ext));

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

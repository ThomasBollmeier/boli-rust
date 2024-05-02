use std::collections::HashMap;

use boli::interpreter::{
    self,
    module_mgmt::{
        extension::{new_extension, new_extension_dir},
        file_system::new_directory,
        module_loader::ModuleLoader,
        ModuleDirRef,
    },
    values::{new_valueref, IntValue, ValueRef, ValueType},
};

#[test]
fn test_load_file_module_ok() {
    let search_dirs: Vec<ModuleDirRef> = vec![new_directory("tests", "code")];

    let module_loader = ModuleLoader::new(&search_dirs);
    let result = module_loader.load_module("core::list");

    assert!(result.is_ok());

    let values = result.unwrap();
    assert_eq!(values.len(), 1);

    let reverse_value = values.get("reverse").unwrap();
    assert_eq!(reverse_value.borrow().get_type(), ValueType::Lambda);
}

#[test]
fn test_load_extension_module_ok() {
    let ext_dir = new_extension_dir("ext");

    let mut values: HashMap<String, ValueRef> = HashMap::new();
    values.insert("answer".to_string(), new_valueref(IntValue { value: 42 }));
    let ext_module = new_extension("q&a", values);
    ext_dir.borrow_mut().add_extension(&ext_module);

    let search_dirs: Vec<ModuleDirRef> = vec![new_directory("tests", "code"), ext_dir];

    let module_loader = ModuleLoader::new(&search_dirs);
    let result = module_loader.load_module("q&a");

    assert!(result.is_ok());

    let values = result.unwrap();
    assert_eq!(values.len(), 1);

    let value = values.get("answer").unwrap();
    assert_eq!(value.borrow().get_type(), ValueType::Int);
}

#[test]
fn test_main_module_ok() {
    let code_dir: ModuleDirRef = new_directory("tests", "code");

    let main_file = code_dir.borrow().get_file("main").unwrap();
    let code = main_file.borrow().read();

    let mut interpreter = interpreter::Interpreter::new();
    interpreter.set_module_search_dirs(&vec![code_dir]);

    let result = interpreter.eval(&code);
    assert!(result.is_ok(), "Error: {:?}", result.err());

    let result = result.unwrap();
    assert_eq!(result.borrow().get_type(), ValueType::List);
    assert_eq!(result.borrow().to_string(), "(list 5 4 3 2 1)");
}

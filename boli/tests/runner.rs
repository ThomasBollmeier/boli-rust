use std::{cell::RefCell, rc::Rc};

use boli::{
    code_repository::{file_system::Directory, CodeDirRef},
    interpreter::{module_loader::ModuleLoader, values::ValueType},
};

#[test]
fn test_module_loader() {
    let search_dirs: Vec<CodeDirRef> = vec![Rc::new(RefCell::new(Directory::new("tests", "code")))];

    let module_loader = ModuleLoader::new(&search_dirs);
    let result = module_loader.load_module("core::list");

    assert!(result.is_ok());

    let values = result.unwrap();
    assert_eq!(values.len(), 1);

    let reverse_value = values.get("reverse").unwrap();
    assert_eq!(reverse_value.borrow().get_type(), ValueType::Lambda);
}

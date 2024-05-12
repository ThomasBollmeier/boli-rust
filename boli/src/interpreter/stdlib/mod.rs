use std::{cell::RefCell, rc::Rc};

use crate::interpreter::module_mgmt::extension::new_extension_dir;

use super::module_mgmt::{extension::Extension, ModuleDirRef};

pub fn create_stdlib() -> ModuleDirRef {
    let stdlib = new_extension_dir("");

    let list_module = Extension::with_code("list", include_str!("list.boli"));
    stdlib
        .borrow_mut()
        .add_extension(&Rc::new(RefCell::new(list_module)));

    stdlib
}

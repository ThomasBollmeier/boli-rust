use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::environment::Environment;
use crate::interpreter::module_mgmt::extension::{new_extension, new_extension_dir, ExtensionRef};
use crate::interpreter::stdlib::load_module_code;

pub fn create_seq_collection_extension(extension_deps: &Vec<ExtensionRef>) -> ExtensionRef {
    let core_env = Environment::new_ref();
    let env = Environment::with_parent(&core_env);
    let collection_env = Rc::new(RefCell::new(env));

    let deps = new_extension_dir("deps");
    for dep in extension_deps {
        deps.borrow_mut().add_extension(dep);
    }

    let mut search_dirs = collection_env.borrow().get_module_search_dirs();
    search_dirs.push(deps);
    Environment::set_module_search_dirs(&collection_env, &search_dirs);

    let values =
        load_module_code(&collection_env, include_str!("seqcol.boli")).unwrap_or(HashMap::new());

    new_extension("seqcol", values)
}

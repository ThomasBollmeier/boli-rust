use std::rc::Rc;

use crate::code_repository::Directory;

pub struct ModuleLoader {
    search_dirs: Vec<Rc<dyn Directory>>,
}

impl ModuleLoader {
    pub fn new(search_dirs: &Vec<Rc<dyn Directory>>) -> Self {
        Self {
            search_dirs: search_dirs.clone(),
        }
    }

    pub fn load_module(&self, _path: &str) -> Result<String, String> {
        todo!("Implement module loader")
    }
}

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::interpreter::{environment::Environment, Interpreter, ValueRef};

use super::{ExtensionModule, ModuleDirectory, ModuleObject, ModuleObjectType};

pub struct ExtensionDir {
    name: String,
    dirs: Vec<ExtensionDirRef>,
    extensions: Vec<ExtensionRef>,
}

pub type ExtensionDirRef = Rc<RefCell<ExtensionDir>>;

pub fn new_extension_dir(name: &str) -> ExtensionDirRef {
    Rc::new(RefCell::new(ExtensionDir::new(name)))
}

impl ExtensionDir {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            dirs: Vec::new(),
            extensions: Vec::new(),
        }
    }

    pub fn add_dir(&mut self, dir: &ExtensionDirRef) {
        self.dirs.push(dir.clone());
    }

    pub fn add_extension(&mut self, extension: &ExtensionRef) {
        self.extensions.push(extension.clone());
    }
}

impl ModuleDirectory for ExtensionDir {
    fn get_dir(&self, name: &str) -> Option<super::ModuleDirRef> {
        for dir in &self.dirs {
            if dir.borrow().name == name {
                return Some(dir.clone());
            }
        }

        None
    }

    fn get_file(&self, _name: &str) -> Option<super::ModuleFileRef> {
        None
    }

    fn get_extension(&self, name: &str) -> Option<super::ExtensionModuleRef> {
        for extension in &self.extensions {
            if extension.borrow().get_name() == name {
                return Some(extension.clone());
            }
        }

        None
    }
}

impl ModuleObject for ExtensionDir {
    fn get_type(&self) -> ModuleObjectType {
        ModuleObjectType::Directory
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Extension {
    name: String,
    values: HashMap<String, ValueRef>,
}

pub type ExtensionRef = Rc<RefCell<Extension>>;

pub fn new_extension(name: &str, values: HashMap<String, ValueRef>) -> ExtensionRef {
    Rc::new(RefCell::new(Extension::new(name, values)))
}

impl Extension {
    pub fn new(name: &str, values: HashMap<String, ValueRef>) -> Self {
        Self {
            name: name.to_string(),
            values,
        }
    }

    pub fn with_code(name: &str, code: &str) -> Self {
        let env = Environment::new_ref();
        let mut interpreter = Interpreter::with_environment(&env);

        match interpreter.eval(code) {
            Ok(_) => {
                let env = env.borrow();
                let values = env.get_exported_values();
                Self::new(name, values)
            }
            Err(_) => Self::new(name, HashMap::new()),
        }
    }
}

impl ExtensionModule for Extension {
    fn get_values(&self) -> HashMap<String, ValueRef> {
        self.values.clone()
    }
}

impl ModuleObject for Extension {
    fn get_type(&self) -> ModuleObjectType {
        ModuleObjectType::Extension
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

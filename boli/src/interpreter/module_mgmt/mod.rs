use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::ValueRef;

pub mod extension;
pub mod file_system;
pub mod module_loader;

#[derive(PartialEq, Debug)]
pub enum ModuleObjectType {
    Directory,
    File,
    Extension,
}

pub trait ModuleObject {
    fn get_type(&self) -> ModuleObjectType;
    fn get_name(&self) -> String;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait ModuleDirectory: ModuleObject {
    fn get_dir(&self, name: &str) -> Option<ModuleDirRef>;
    fn get_file(&self, name: &str) -> Option<ModuleFileRef>;
    fn get_extension(&self, name: &str) -> Option<ExtensionModuleRef>;
}

pub type ModuleDirRef = Rc<RefCell<dyn ModuleDirectory>>;

pub trait ModuleFile: ModuleObject {
    fn read(&self) -> String;
}

pub type ModuleFileRef = Rc<RefCell<dyn ModuleFile>>;

pub trait ExtensionModule: ModuleObject {
    fn get_values(&self) -> HashMap<String, ValueRef>;
}

pub type ExtensionModuleRef = Rc<RefCell<dyn ExtensionModule>>;

use std::{cell::RefCell, rc::Rc};

pub mod file_system;

#[derive(PartialEq, Debug)]
pub enum CodeRepoObjectType {
    Directory,
    File,
}

pub trait CodeRepoObject {
    fn get_type(&self) -> CodeRepoObjectType;
    fn get_name(&self) -> String;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait CodeDirectory: CodeRepoObject {
    fn get_dir(&self, name: &str) -> Option<CodeDirRef>;
    fn get_file(&self, name: &str) -> Option<CodeFileRef>;
}

pub type CodeDirRef = Rc<RefCell<dyn CodeDirectory>>;

pub trait CodeFile: CodeRepoObject {
    fn read(&self) -> String;
}

pub type CodeFileRef = Rc<RefCell<dyn CodeFile>>;

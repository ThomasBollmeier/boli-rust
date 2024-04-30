use std::{collections::HashMap, rc::Rc};

pub enum CodeRepoObjectType {
    Directory,
    File,
}

pub trait CodeRepoObject {
    fn get_type(&self) -> CodeRepoObjectType;
    fn get_name(&self) -> String;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait Directory: CodeRepoObject {
    fn list(&self) -> HashMap<String, Rc<dyn CodeRepoObject>>;
}

pub trait File: CodeRepoObject {
    fn read(&self) -> String;
}

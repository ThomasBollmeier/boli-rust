use super::{environment::EnvironmentRef, InterpreterError, ValueRef};
use crate::{code_repository::CodeDirRef, interpreter};
use std::collections::HashMap;

pub struct ModuleLoader {
    search_dirs: Vec<CodeDirRef>,
}

impl ModuleLoader {
    pub fn new(search_dirs: &Vec<CodeDirRef>) -> Self {
        Self {
            search_dirs: search_dirs.clone(),
        }
    }

    pub fn load_module(&self, path: &str) -> Result<HashMap<String, ValueRef>, InterpreterError> {
        for dir in &self.search_dirs {
            match self.load_module_in_dir(dir, path) {
                Ok(env) => return Ok(env.borrow().get_exported_values()),
                Err(_) => continue,
            }
        }

        Err(InterpreterError::new(&format!(
            "module '{}' not found",
            path
        )))
    }

    fn load_module_in_dir(
        &self,
        dir: &CodeDirRef,
        path: &str,
    ) -> Result<EnvironmentRef, InterpreterError> {
        let path_segments = path.split("::").collect::<Vec<&str>>();

        if path_segments.len() == 0 {
            return Err(InterpreterError::new("empty module path"));
        }

        if path_segments.len() == 1 {
            let module_name = path_segments[0];
            match dir.borrow().get_file(module_name) {
                Some(module_file) => {
                    let module_code = module_file.borrow().read();
                    let mut interpreter = interpreter::Interpreter::new();
                    interpreter.eval(&module_code)?;
                    return Ok(interpreter.env.clone());
                }
                None => {
                    return Err(InterpreterError::new(&format!(
                        "module '{}' not found",
                        module_name
                    )))
                }
            };
        }

        let dir_name = path_segments[0];

        match dir.borrow().get_dir(&dir_name) {
            Some(sub_dir) => {
                return self.load_module_in_dir(&sub_dir, &path_segments[1..].join("::"));
            }
            None => {
                return Err(InterpreterError::new(&format!(
                    "directory '{}' not found",
                    dir_name
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::code_repository::{
        CodeDirectory, CodeFile, CodeFileRef, CodeRepoObject, CodeRepoObjectType,
    };
    use std::{cell::RefCell, rc::Rc};

    type DirRef = Rc<RefCell<TestDir>>;
    type FileRef = Rc<RefCell<TestFile>>;

    struct TestDir {
        name: String,
        dirs: Vec<DirRef>,
        files: Vec<FileRef>,
    }

    impl TestDir {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                dirs: Vec::new(),
                files: Vec::new(),
            }
        }

        fn add_dir(&mut self, dir: &DirRef) {
            self.dirs.push(dir.clone());
        }

        fn add_file(&mut self, file: &FileRef) {
            self.files.push(file.clone());
        }
    }

    impl CodeDirectory for TestDir {
        fn get_dir(&self, name: &str) -> Option<CodeDirRef> {
            for dir in &self.dirs {
                if dir.borrow().name == name {
                    return Some(dir.clone());
                }
            }

            None
        }

        fn get_file(&self, name: &str) -> Option<CodeFileRef> {
            for file in &self.files {
                if file.borrow().name == name {
                    return Some(file.clone());
                }
            }

            None
        }
    }

    impl CodeRepoObject for TestDir {
        fn get_type(&self) -> CodeRepoObjectType {
            CodeRepoObjectType::Directory
        }

        fn get_name(&self) -> String {
            self.name.clone()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    struct TestFile {
        name: String,
        content: String,
    }

    impl TestFile {
        fn new(name: &str, content: &str) -> Self {
            Self {
                name: name.to_string(),
                content: content.to_string(),
            }
        }
    }

    impl CodeFile for TestFile {
        fn read(&self) -> String {
            self.content.clone()
        }
    }

    impl CodeRepoObject for TestFile {
        fn get_type(&self) -> CodeRepoObjectType {
            CodeRepoObjectType::File
        }

        fn get_name(&self) -> String {
            self.name.clone()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn load_module_ok() {
        let current_dir = Rc::new(RefCell::new(TestDir::new("")));
        let core_dir = Rc::new(RefCell::new(TestDir::new("core")));
        let list_module = Rc::new(RefCell::new(TestFile::new(
            "list",
            r#"
        (def (reverse l)
            (def (helper l acc)
                (if (empty? l)
                    acc
                    (helper 
                        (tail l) 
                        (cons 
                            (head l) 
                            acc))))
            (helper l '()))
        "#,
        )));
        core_dir.borrow_mut().add_file(&list_module);
        current_dir.borrow_mut().add_dir(&core_dir);

        let loader = ModuleLoader::new(&vec![current_dir.clone()]);

        let loaded_values = loader.load_module("core::list").unwrap();

        assert_eq!(loaded_values.len(), 1);

        let reverse_value = loaded_values.get("reverse").unwrap();

        assert_eq!(
            reverse_value.borrow().get_type(),
            interpreter::ValueType::Lambda
        );
    }
}

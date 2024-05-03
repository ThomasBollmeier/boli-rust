use crate::interpreter::{
    borrow_value, downcast_value, environment::EnvironmentRef, misc_functions::OutputRef,
    module_mgmt::ModuleDirRef, new_valueref, Callable, Interpreter, InterpreterError, NilValue,
    SymbolValue, ValueRef,
};
use std::collections::HashMap;

pub struct ModuleLoader {
    search_dirs: Vec<ModuleDirRef>,
    output: OutputRef,
}

impl ModuleLoader {
    pub fn new(search_dirs: &Vec<ModuleDirRef>, output: &OutputRef) -> Self {
        Self {
            search_dirs: search_dirs.clone(),
            output: output.clone(),
        }
    }

    pub fn load_module(&self, path: &str) -> Result<HashMap<String, ValueRef>, InterpreterError> {
        let mut load_error: Option<InterpreterError> = None;

        for dir in &self.search_dirs {
            match self.load_module_in_dir(dir, path) {
                Ok(env) => return Ok(env),
                Err(err) => {
                    load_error = Some(err);
                    continue;
                }
            }
        }

        match load_error {
            Some(err) => Err(err),
            None => Err(InterpreterError::new(&format!(
                "module '{}' not found",
                path
            ))),
        }
    }

    fn load_module_in_dir(
        &self,
        dir: &ModuleDirRef,
        path: &str,
    ) -> Result<HashMap<String, ValueRef>, InterpreterError> {
        let path_segments = path.split("::").collect::<Vec<&str>>();

        if path_segments.len() == 0 {
            return Err(InterpreterError::new("empty module path"));
        }

        if path_segments.len() == 1 {
            let module_name = path_segments[0];
            let module_file = module_name.to_string() + ".boli";

            if let Some(module_file) = dir.borrow().get_file(&module_file) {
                let module_code = module_file.borrow().read();
                let mut interpreter = Interpreter::new();
                interpreter.set_module_search_dirs(&self.search_dirs);
                interpreter.redirect_output(&self.output);
                interpreter.eval(&module_code)?;
                return Ok(interpreter.env.clone().borrow().get_exported_values());
            }

            if let Some(ext_module) = dir.borrow().get_extension(module_name) {
                return Ok(ext_module.borrow().get_values());
            }

            return Err(InterpreterError::new(&format!(
                "module '{}' not found",
                module_name
            )));
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

pub struct RequireFn {
    env: EnvironmentRef,
    module_loader: ModuleLoader,
}

impl RequireFn {
    pub fn new(env: &EnvironmentRef, search_dirs: &Vec<ModuleDirRef>, output: &OutputRef) -> Self {
        Self {
            env: env.clone(),
            module_loader: ModuleLoader::new(search_dirs, output),
        }
    }
}

impl Callable for RequireFn {
    fn call(&self, args: &Vec<ValueRef>) -> Result<ValueRef, InterpreterError> {
        if args.len() != 1 {
            return Err(InterpreterError::new(
                "require function expects exactly one argument",
            ));
        }

        let arg0 = &borrow_value(&args[0]);
        let module_path = downcast_value::<SymbolValue>(arg0);
        if module_path.is_none() {
            return Err(InterpreterError::new(
                "require function expects a symbol as the first argument",
            ));
        }
        let module_path = module_path.unwrap().value.clone();
        let module_imports = self.module_loader.load_module(&module_path)?;

        self.env.borrow_mut().import_values(module_imports);

        Ok(new_valueref(NilValue {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::{
        self,
        misc_functions::StdOutput,
        module_mgmt::{
            extension::{Extension, ExtensionDir},
            ModuleDirectory, ModuleFile, ModuleFileRef, ModuleObject, ModuleObjectType,
        },
        new_valueref, IntValue,
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

    impl ModuleDirectory for TestDir {
        fn get_dir(&self, name: &str) -> Option<ModuleDirRef> {
            for dir in &self.dirs {
                if dir.borrow().name == name {
                    return Some(dir.clone());
                }
            }

            None
        }

        fn get_file(&self, name: &str) -> Option<ModuleFileRef> {
            for file in &self.files {
                if file.borrow().name == name {
                    return Some(file.clone());
                }
            }

            None
        }

        fn get_extension(
            &self,
            _name: &str,
        ) -> Option<interpreter::module_mgmt::ExtensionModuleRef> {
            None
        }
    }

    impl ModuleObject for TestDir {
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

    impl ModuleFile for TestFile {
        fn read(&self) -> String {
            self.content.clone()
        }
    }

    impl ModuleObject for TestFile {
        fn get_type(&self) -> ModuleObjectType {
            ModuleObjectType::File
        }

        fn get_name(&self) -> String {
            self.name.clone()
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn load_file_ok() {
        let current_dir = Rc::new(RefCell::new(TestDir::new("")));
        let core_dir = Rc::new(RefCell::new(TestDir::new("core")));
        current_dir.borrow_mut().add_dir(&core_dir);

        let list_module = Rc::new(RefCell::new(TestFile::new(
            "list.boli",
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

        let output: OutputRef = Rc::new(RefCell::new(StdOutput::new()));

        let loader = ModuleLoader::new(&vec![current_dir.clone()], &output);

        let loaded_values = loader.load_module("core::list").unwrap();

        assert_eq!(loaded_values.len(), 1);

        let reverse_value = loaded_values.get("reverse").unwrap();

        assert_eq!(
            reverse_value.borrow().get_type(),
            interpreter::ValueType::Lambda
        );
    }

    #[test]
    fn load_extension_ok() {
        let search_dir = Rc::new(RefCell::new(ExtensionDir::new("")));
        let ext_dir = Rc::new(RefCell::new(ExtensionDir::new("ext")));
        search_dir.borrow_mut().add_dir(&ext_dir);

        let mut values: HashMap<String, ValueRef> = HashMap::new();
        values.insert("answer".to_string(), new_valueref(IntValue { value: 42 }));

        let ext_module = Rc::new(RefCell::new(Extension::new("q&a", values)));
        ext_dir.borrow_mut().add_extension(&ext_module);

        let output: OutputRef = Rc::new(RefCell::new(StdOutput::new()));

        let loader = ModuleLoader::new(&vec![search_dir.clone()], &output);

        let loaded_values = loader.load_module("ext::q&a").unwrap();

        assert_eq!(loaded_values.len(), 1);

        let answer = loaded_values.get("answer").unwrap();
        assert_eq!(answer.borrow().get_type(), interpreter::ValueType::Int);

        let answer = answer.borrow();
        let answer_value = answer.as_any().downcast_ref::<IntValue>().unwrap();
        assert_eq!(answer_value.value, 42);
    }
}

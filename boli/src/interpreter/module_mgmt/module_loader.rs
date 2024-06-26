use crate::{
    frontend::lexer::tokens::TokenType,
    interpreter::{
        borrow_value, downcast_value,
        environment::{EnvironmentBuilder, EnvironmentRef},
        error,
        module_mgmt::ModuleDirRef,
        new_valueref, Callable, Interpreter, InterpreterError, NilValue, QuoteValue, SymbolValue,
        ValueRef, VectorValue,
    },
};
use std::collections::HashMap;

pub struct ModuleLoader {
    env: EnvironmentRef,
}

impl ModuleLoader {
    pub fn new(env: &EnvironmentRef) -> Self {
        Self { env: env.clone() }
    }

    pub fn load_module(&self, path: &str) -> Result<HashMap<String, ValueRef>, InterpreterError> {
        let mut load_error: Option<InterpreterError> = None;
        let module_search_dirs = self.env.borrow().get_module_search_dirs();

        for dir in &module_search_dirs {
            match self.load_module_in_dir(dir, path) {
                Ok(value_map) => return Ok(value_map),
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
                let module_env = EnvironmentBuilder::new().parent(&self.env).build();
                let mut interpreter = Interpreter::with_environment(&module_env);
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
}

impl RequireFn {
    pub fn new(env: &EnvironmentRef) -> Self {
        Self { env: env.clone() }
    }
}

impl Callable for RequireFn {
    fn call(&self, args: &Vec<ValueRef>) -> Result<ValueRef, InterpreterError> {
        let num_args = args.len();

        if num_args != 1 && num_args != 2 {
            return Err(InterpreterError::new(
                "require function expects 1-2 arguments",
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

        let module_loader = ModuleLoader::new(&self.env);
        let module_imports = module_loader.load_module(&module_path)?;

        if num_args == 2 {
            let arg1 = &borrow_value(&args[1]);
            let alias = downcast_value::<SymbolValue>(arg1);
            if alias.is_none() {
                return Err(InterpreterError::new(
                    "require function expects a symbol as the second argument",
                ));
            }
            let alias = alias.unwrap().value.clone();

            self.env
                .borrow_mut()
                .import_values_with_alias(module_imports, &alias);
        } else {
            self.env.borrow_mut().import_values(module_imports);
        }

        Ok(new_valueref(NilValue {}))
    }
}

pub struct ProvideFn {
    env: EnvironmentRef,
}

impl ProvideFn {
    pub fn new(env: &EnvironmentRef) -> Self {
        Self { env: env.clone() }
    }
}

impl Callable for ProvideFn {
    fn call(&self, args: &Vec<ValueRef>) -> Result<ValueRef, InterpreterError> {
        let num_args = args.len();

        if num_args != 1 {
            return Err(InterpreterError::new(
                "provide function expects one argument",
            ));
        }

        let arg0 = &borrow_value(&args[0]);
        let arg0 = downcast_value::<VectorValue>(arg0);
        if arg0.is_none() {
            return error("provide expects a list as the argument");
        }
        let export_list = arg0.unwrap();

        for export_name in export_list.elements.iter() {
            let export_name = &borrow_value(export_name);
            let export_name = downcast_value::<QuoteValue>(export_name);
            if export_name.is_none() {
                return error("provide expects a list of symbols as the argument");
            }
            let export_token = &export_name.unwrap().token;
            if export_token.token_type != TokenType::Identifier {
                return error("provide expects a list of quoted identifiers as the argument");
            }

            let export_name = export_token.get_string_value().unwrap();
            self.env.borrow_mut().export(&export_name);
        }

        Ok(new_valueref(NilValue {}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::{
        self,
        environment::EnvironmentBuilder,
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

        let env = EnvironmentBuilder::new()
            .search_dirs(&vec![current_dir])
            .build();

        let loader = ModuleLoader::new(&env);

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

        let env = EnvironmentBuilder::new()
            .search_dirs(&vec![search_dir])
            .build();

        let loader = ModuleLoader::new(&env);

        let loaded_values = loader.load_module("ext::q&a").unwrap();

        assert_eq!(loaded_values.len(), 1);

        let answer = loaded_values.get("answer").unwrap();
        assert_eq!(answer.borrow().get_type(), interpreter::ValueType::Int);

        let answer = answer.borrow();
        let answer_value = answer.as_any().downcast_ref::<IntValue>().unwrap();
        assert_eq!(answer_value.value, 42);
    }

    #[test]
    fn load_module_with_explicit_exports() {
        let current_dir = Rc::new(RefCell::new(TestDir::new("")));
        let core_dir = Rc::new(RefCell::new(TestDir::new("core")));
        current_dir.borrow_mut().add_dir(&core_dir);

        let list_module = Rc::new(RefCell::new(TestFile::new(
            "list.boli",
            r#"
        (provide '(reverse))
        
        (def (helper l acc)
            (if (empty? l)
                acc
                (helper (tail l) 
                        (cons 
                            (head l) 
                            acc))))
        (def (reverse l)
            (helper l '()))
        "#,
        )));
        core_dir.borrow_mut().add_file(&list_module);

        let env = EnvironmentBuilder::new()
            .search_dirs(&vec![current_dir])
            .build();

        let loader = ModuleLoader::new(&env);

        let loaded_values = loader.load_module("core::list").unwrap();

        assert_eq!(loaded_values.len(), 1);

        let reverse_value = loaded_values.get("reverse").unwrap();

        assert_eq!(
            reverse_value.borrow().get_type(),
            interpreter::ValueType::Lambda
        );

        let helper = env.borrow().get("helper");
        assert!(helper.is_none());
    }
}

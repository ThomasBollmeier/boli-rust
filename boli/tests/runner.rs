use std::{cell::RefCell, collections::HashMap, rc::Rc};

use boli::interpreter::{
    self,
    environment::EnvironmentBuilder,
    misc_functions::OutputRef,
    module_mgmt::{
        extension::{new_extension, new_extension_dir},
        file_system::new_directory,
        module_loader::ModuleLoader,
        ModuleDirRef,
    },
    values::{new_valueref, IntValue, ValueRef, ValueType},
};

#[test]
fn test_load_extension_module_ok() {
    let ext_dir = new_extension_dir("ext");

    let mut values: HashMap<String, ValueRef> = HashMap::new();
    values.insert("answer".to_string(), new_valueref(IntValue { value: 42 }));
    let ext_module = new_extension("q&a", values);
    ext_dir.borrow_mut().add_extension(&ext_module);

    let search_dirs: Vec<ModuleDirRef> = vec![new_directory("tests", "code"), ext_dir];
    let output: OutputRef = Rc::new(RefCell::new(StringOutput::new()));

    let env = EnvironmentBuilder::new()
        .search_dirs(&search_dirs)
        .output(&output)
        .build();

    let module_loader = ModuleLoader::new(&env);
    let result = module_loader.load_module("q&a");

    assert!(result.is_ok());

    let values = result.unwrap();
    assert_eq!(values.len(), 1);

    let value = values.get("answer").unwrap();
    assert_eq!(value.borrow().get_type(), ValueType::Int);
}

#[derive(Debug)]
struct StringOutput {
    output: String,
}

impl StringOutput {
    fn new() -> Self {
        Self {
            output: String::new(),
        }
    }

    fn get_output(&self) -> &str {
        &self.output
    }
}

impl interpreter::misc_functions::Output for StringOutput {
    fn print(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn read_expected_output_file(file_name: &str) -> String {
    let path = format!("tests/output/{}", file_name);
    std::fs::read_to_string(path).unwrap()
}

fn run_file(input_file_name: &str, expected_output_file: &str) {
    let code_dir: ModuleDirRef = new_directory("tests", "input");
    let output: OutputRef = Rc::new(RefCell::new(StringOutput::new()));

    let env = EnvironmentBuilder::new()
        .search_dirs(&vec![code_dir.clone()])
        .output(&output)
        .with_stdlib(true)
        .build();

    let file = code_dir.borrow().get_file(input_file_name).unwrap();
    let code = file.borrow().read();

    let mut interpreter = interpreter::Interpreter::with_environment(&env);

    let result = interpreter.eval(&code);
    assert!(result.is_ok(), "Error: {:?}", result.err());

    let str_out = output.borrow();
    let str_out = str_out.as_any().downcast_ref::<StringOutput>().unwrap();

    let actual_output = str_out.get_output();
    let expected_output = read_expected_output_file(expected_output_file);

    assert_eq!(actual_output, expected_output);
}

#[test]
fn test_hello() {
    run_file("hello.boli", "hello.out");
}

#[test]
fn test_reverse() {
    run_file("reverse.boli", "reverse.out");
}

#[test]
fn test_varargs() {
    run_file("varargs.boli", "varargs.out");
}

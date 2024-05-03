pub mod count_functions;
pub mod environment;
pub mod list_functions;
pub mod misc_functions;
pub mod module_mgmt;
pub mod number_functions;
pub mod string_functions;
pub mod struct_functions;
pub mod values;

use std::cell::RefCell;
use std::rc::Rc;
use std::vec;

use crate::frontend::lexer::tokens::{LogicalOp, Op};
use crate::frontend::parser::{ast::*, Parser};
use environment::Environment;

use values::*;

use self::environment::EnvironmentRef;
use self::misc_functions::{DisplayLn, Display_, OutputRef, StdOutput, Write, WriteLn};
use self::module_mgmt::file_system::new_directory;
use self::module_mgmt::module_loader::RequireFn;
use self::module_mgmt::ModuleDirRef;

pub struct Interpreter {
    pub stack: Vec<EvalResult>,
    pub env: EnvironmentRef,
    module_search_dirs: Vec<ModuleDirRef>,
    output: OutputRef,
}

impl Interpreter {
    pub fn new() -> Self {
        let current_dir = new_directory(".", "");
        Self {
            stack: Vec::new(),
            env: Rc::new(RefCell::new(Environment::new())),
            module_search_dirs: vec![current_dir],
            output: Rc::new(RefCell::new(StdOutput::new())),
        }
    }

    pub fn with_environment(env: &Rc<RefCell<Environment>>) -> Self {
        let current_dir = new_directory(".", "");
        Self {
            stack: Vec::new(),
            env: env.clone(),
            module_search_dirs: vec![current_dir],
            output: Rc::new(RefCell::new(StdOutput::new())),
        }
    }

    pub fn configure(&mut self, dirs: Option<Vec<ModuleDirRef>>, output: Option<OutputRef>) {
        if dirs.is_none() && output.is_none() {
            return;
        }

        if let Some(dirs) = dirs {
            self.module_search_dirs = dirs.clone();
        }

        let mut output_changed = false;

        if let Some(output) = &output {
            self.output = output.clone();
            output_changed = true;
        }

        let require_fn = RequireFn::new(&self.env, &self.module_search_dirs, &self.output);
        self.env
            .borrow_mut()
            .set_builtin("require", &Rc::new(require_fn));

        if !output_changed {
            return;
        }

        let output = output.unwrap();

        self.env
            .borrow_mut()
            .set_builtin("write", &Rc::new(Write::with_output(&output)));
        self.env
            .borrow_mut()
            .set_builtin("writeln", &Rc::new(WriteLn::with_output(&output)));
        self.env
            .borrow_mut()
            .set_builtin("display", &Rc::new(Display_::with_output(&output)));
        self.env
            .borrow_mut()
            .set_builtin("displayln", &Rc::new(DisplayLn::with_output(&output)));
    }

    pub fn eval(&mut self, code: &str) -> EvalResult {
        let parser = Parser::new();
        let program: AstRef = new_astref(
            parser
                .parse(code)
                .map_err(|e| InterpreterError::new(&e.message))?,
        );

        self.eval_ast(&program)
    }

    fn new_eval_error(&mut self, message: &str) -> EvalResult {
        Err(InterpreterError::new(message))
    }

    fn eval_ast(&mut self, ast: &AstRef) -> EvalResult {
        ast.borrow().accept(self);
        self.stack
            .pop()
            .unwrap_or(self.new_eval_error("No value on the stack"))
    }

    fn eval_block(&mut self, children: &Vec<AstRef>) -> EvalResult {
        let mut result: EvalResult = Ok(new_valueref(NilValue {}));

        for child in children {
            result = self.eval_ast(child);
            if result.is_err() {
                return result;
            }
        }

        result
    }

    fn is_truthy(&self, value: &ValueRef) -> bool {
        let value = &borrow_value(value);
        match value.get_type() {
            ValueType::Nil => false,
            ValueType::Bool => {
                let bool_value = downcast_value::<BoolValue>(value).unwrap();
                bool_value.value
            }
            ValueType::Int => {
                let int_value = downcast_value::<IntValue>(value).unwrap();
                int_value.value != 0
            }
            ValueType::List => {
                let list_value = downcast_value::<ListValue>(value).unwrap();
                !list_value.elements.is_empty()
            }
            _ => true,
        }
    }
}

impl AstVisitor for Interpreter {
    fn visit_program(&mut self, program: &Program) {
        let result = self.eval_block(&program.children);
        self.stack.push(result);
    }

    fn visit_block(&mut self, block: &Block) {
        let env = self.env.clone();
        self.env = Rc::new(RefCell::new(Environment::with_parent(&self.env)));

        let result = self.eval_block(&block.children);
        self.stack.push(result);
        self.env = env;
    }

    fn visit_integer(&mut self, integer: &Integer) {
        self.stack.push(Ok(new_valueref(IntValue {
            value: integer.value,
        })));
    }

    fn visit_real(&mut self, real: &Real) {
        self.stack
            .push(Ok(new_valueref(RealValue { value: real.value })));
    }

    fn visit_bool(&mut self, bool: &Bool) {
        self.stack
            .push(Ok(new_valueref(BoolValue { value: bool.value })));
    }

    fn visit_str(&mut self, str: &Str) {
        self.stack.push(Ok(new_valueref(StrValue {
            value: str.value.clone(),
        })));
    }

    fn visit_nil(&mut self) {
        self.stack.push(Ok(new_valueref(NilValue {})));
    }

    fn visit_identifier(&mut self, identifier: &Identifier) {
        let value = self.env.borrow().get(&identifier.value);
        if value.is_none() {
            let err = self.new_eval_error(&format!("Undefined identifier: {}", identifier.value));
            self.stack.push(err);
            return;
        }

        self.stack.push(Ok(value.unwrap().clone()));
    }

    fn visit_absolute_name(&mut self, absolute_name: &AbsoluteName) {
        let abs_name = absolute_name.segments.join("::");
        let value = self.env.borrow().get(&abs_name);
        if value.is_none() {
            let err = self.new_eval_error(&format!("Undefined identifier: {}", abs_name));
            self.stack.push(err);
            return;
        }

        self.stack.push(Ok(value.unwrap().clone()));
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        self.stack
            .push(Ok(new_valueref(SymbolValue::new(&symbol.value[1..]))));
    }

    fn visit_quote(&mut self, quote: &Quote) {
        self.stack.push(Ok(new_valueref(QuoteValue {
            token: quote.value.clone(),
        })));
    }

    fn visit_operator(&mut self, operator: &Operator) {
        match operator.value {
            Op::Plus => {
                let add = self.env.borrow().get("+").unwrap();
                self.stack.push(Ok(add.clone()));
            }
            Op::Minus => {
                let sub = self.env.borrow().get("-").unwrap();
                self.stack.push(Ok(sub.clone()));
            }
            Op::Asterisk => {
                let mul = self.env.borrow().get("*").unwrap();
                self.stack.push(Ok(mul.clone()));
            }
            Op::Slash => {
                let div = self.env.borrow().get("/").unwrap();
                self.stack.push(Ok(div.clone()));
            }
            Op::Caret => {
                let pow = self.env.borrow().get("^").unwrap();
                self.stack.push(Ok(pow.clone()));
            }
            Op::Percent => {
                let mod_ = self.env.borrow().get("%").unwrap();
                self.stack.push(Ok(mod_.clone()));
            }
        }
    }

    fn visit_logical_operator(&mut self, operator: &LogicalOperator) {
        match operator.value {
            LogicalOp::Eq => {
                let eq = self.env.borrow().get("=").unwrap();
                self.stack.push(Ok(eq.clone()));
            }
            LogicalOp::Gt => {
                let gt = self.env.borrow().get(">").unwrap();
                self.stack.push(Ok(gt.clone()));
            }
            LogicalOp::Ge => {
                let ge = self.env.borrow().get(">=").unwrap();
                self.stack.push(Ok(ge.clone()));
            }
            LogicalOp::Lt => {
                let lt = self.env.borrow().get("<").unwrap();
                self.stack.push(Ok(lt.clone()));
            }
            LogicalOp::Le => {
                let le = self.env.borrow().get("<=").unwrap();
                self.stack.push(Ok(le.clone()));
            }
        }
    }

    fn visit_list(&mut self, list: &List) {
        let mut elements = vec![];

        for element in &list.elements {
            let elem_result = self.eval_ast(element);
            if elem_result.is_err() {
                self.stack.push(elem_result);
                return;
            }
            elements.push(elem_result.unwrap());
        }

        self.stack.push(Ok(new_valueref(ListValue { elements })));
    }

    fn visit_def(&mut self, def: &Definition) {
        let name = def.name.clone();
        let value = self.eval_ast(&def.value);

        if value.is_err() {
            self.stack.push(value);
            return;
        }

        self.env.borrow_mut().set(name, value.unwrap().clone());
        self.stack.push(Ok(new_valueref(NilValue {})));
    }

    fn visit_struct_def(&mut self, struct_def: &StructDefinition) {
        let struct_type = new_valueref(StructTypeValue::new(&struct_def.name, &struct_def.fields));
        self.env
            .borrow_mut()
            .set(struct_def.name.clone(), struct_type.clone());

        let create_struct_name = format!("create-{}", &struct_def.name);
        self.env.borrow_mut().set_builtin(
            &create_struct_name,
            &Rc::new(CreateStructValue::new(&struct_type)),
        );

        for field in &struct_def.fields {
            let getter_name = format!("{}-{}", &struct_def.name, &field);
            self.env
                .borrow_mut()
                .set_builtin(&getter_name, &Rc::new(GetStructField::new(&field)));

            let setter_name = format!("{}-set-{}!", &struct_def.name, &field);
            self.env
                .borrow_mut()
                .set_builtin(&setter_name, &Rc::new(SetStructField::new(&field)));
        }

        self.stack.push(Ok(new_valueref(NilValue {})));
    }

    fn visit_if(&mut self, if_expr: &IfExpression) {
        let condition = self.eval_ast(&if_expr.condition);
        if condition.is_err() {
            self.stack.push(condition);
            return;
        }
        let condition = condition.unwrap();

        let result = if self.is_truthy(&condition) {
            self.eval_ast(&if_expr.consequent)
        } else {
            self.eval_ast(&if_expr.alternate)
        };

        self.stack.push(result);
    }

    fn visit_lambda(&mut self, lambda: &Lambda) {
        let lambda_value = new_valueref(LambdaValue {
            name: lambda.name.clone(),
            parameters: lambda.parameters.clone(),
            variadic: lambda.variadic.clone(),
            body: lambda.body.clone(),
            env: self.env.clone(),
        });
        self.stack.push(Ok(lambda_value));
    }

    fn visit_call(&mut self, call: &Call) {
        let callee = self.eval_ast(&call.callee);
        if callee.is_err() {
            self.stack.push(callee);
            return;
        }

        let callee = callee.unwrap();
        let callee = &borrow_value(&callee);
        let callee_type = callee.get_type();

        let callable: &dyn Callable = match callee_type {
            ValueType::Lambda => downcast_value::<LambdaValue>(&callee).unwrap(),
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(&callee).unwrap(),
            _ => {
                let err = self.new_eval_error("Callee is not a function");
                self.stack.push(err);
                return;
            }
        };

        let mut args = vec![];
        for arg in &call.arguments {
            let arg = self.eval_ast(arg);
            if arg.is_err() {
                self.stack.push(arg);
                return;
            }
            args.push(arg.unwrap());
        }

        if call.is_tail_call {
            let tail_call = new_valueref(TailCallValue {
                arguments: args.clone(),
            });
            self.stack.push(Ok(tail_call));
            return;
        }

        loop {
            let result = callable.call(&args);

            match result {
                Ok(result) => {
                    {
                        let res = &borrow_value(&result);
                        if res.get_type() == ValueType::TailCall {
                            let tail_call = downcast_value::<TailCallValue>(&res).unwrap();
                            args = tail_call.arguments.clone();
                            continue;
                        }
                    }
                    self.stack.push(Ok(result));
                    return;
                }
                Err(_) => {
                    self.stack.push(result);
                    return;
                }
            }
        }
    }

    fn visit_spread_expr(&mut self, _spread_expr: &SpreadExpr) {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_eval_bool() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("#t").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#true");
    }

    #[test]
    fn test_eval_integer() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("42").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_real() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("42,0").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Real);
        assert_eq!(result.to_string(), "42,0");
    }

    #[test]
    fn test_eval_string() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("\"Hello, World!\"").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Str);
        assert_eq!(result.to_string(), "\"Hello, World!\"");
    }

    #[test]
    fn test_eval_list() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("'(1 2 3 (4 5))").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::List);
        assert_eq!(result.to_string(), "(list 1 2 3 (list 4 5))");
    }

    #[test]
    fn test_eval_addition() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(+ 1 2 3 4,0)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Real);
        assert_eq!(result.to_string(), "10,0");
    }

    #[test]
    fn test_eval_subtraction() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(- 44 1 1)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_multiplication() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(* 2 3 7)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_division() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(/ 84,5 2)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Real);
        assert_eq!(result.to_string(), "42,25");
    }

    #[test]
    fn test_eval_power() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(^ 2 2 3)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "256");
    }

    #[test]
    fn test_eval_modulo() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(% 85 43)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_eq() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(= 42 (- 43 1))").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#true");
    }

    #[test]
    fn test_eval_gt() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(> 43 42 41,0)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#true");
    }

    #[test]
    fn test_eval_ge() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(>= 43 42 42)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#true");
    }

    #[test]
    fn test_eval_lt() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(< 41,0 42 43)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#true");
    }

    #[test]
    fn test_eval_le() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(<= 42 42 43)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#true");
    }

    #[test]
    fn test_eval_if() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(if #t 42 43)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");

        let result = interpreter.eval("(if #f 43 42)").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_identifier() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def answer 42)
            answer
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_function_call() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def (fac n)
                (if (= n 0)
                    1
                    (* n (fac (- n 1)))))
            (fac 5)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "120");
    }

    #[test]
    fn test_eval_tailrec() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def (fac n)
                (def (fac-iter n acc)
                    (if (= n 0)
                        acc
                        (fac-iter (- n 1) (* n acc))))
                (fac-iter n 1))
            (fac 5)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "120");
    }

    #[test]
    fn test_eval_quote() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            '((+ 1 2) (a b))
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::List);
        assert_eq!(result.to_string(), "(list (list '+ 1 2) (list 'a 'b))");
    }

    #[test]
    fn test_create_struct() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def-struct person (name first-name))
            (def philosophus (create-person "Nietzsche" "Friedrich"))
            philosophus
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Struct);
        assert_eq!(
            result.to_string(),
            r#"(struct person 'name "Nietzsche" 'first-name "Friedrich")"#
        );
    }

    #[test]
    fn test_get_struct_field() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def-struct person (name first-name))
            (def philosophus (create-person "Nietzsche" "Friedrich"))
            (person-name philosophus)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Str);
        assert_eq!(result.to_string(), "\"Nietzsche\"");
    }

    #[test]
    fn test_set_struct_field() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def-struct person (name first-name))
            (def ego (create-person "Bollmeier" "Thomas"))
            (person-set-first-name! ego "Tom")
            ego
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Struct);
        assert_eq!(
            result.to_string(),
            r#"(struct person 'name "Bollmeier" 'first-name "Tom")"#
        );
    }

    #[test]
    fn test_struct_getter() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def-struct date (year month day))
            (def-struct person (name first-name birth-date))
            (def ego (create-person "Bollmeier" "Thomas" (create-date 1966 7 11)))
            (struct-get ego '(birth-date year))
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "1966");
    }
    #[test]

    fn test_struct_setter() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def-struct date (year month day))
            (def-struct person (name first-name birth-date))
            (def ego (create-person "Bollmeier" "Thomas" (create-date 1970 7 11)))
            (struct-set! ego '(birth-date year) 1966)
            ego
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Struct);
        assert_eq!(result.to_string(), "(struct person 'name \"Bollmeier\" 'first-name \"Thomas\" 'birth-date (struct date 'year 1966 'month 7 'day 11))");
    }

    #[test]
    fn test_create_hash_table() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def options (create-hash-table))
            (struct-set! options '(action) "parse")
            (struct-set! options '(input-file) "code.boli")
            options
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Struct);
        assert_eq!(
            result.to_string(),
            "(hash-table 'action \"parse\" 'input-file \"code.boli\")"
        );
    }

    #[test]
    fn test_eval_symbol() {
        let mut interpreter = Interpreter::new();
        let code = r#" 'symbol "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Symbol);
        assert_eq!(result.to_string(), "'symbol");
    }
}

pub mod environment;
pub mod misc_functions;
pub mod module_mgmt;
pub mod number_functions;
pub mod prelude;
pub mod struct_functions;
pub mod values;

use std::rc::Rc;
use std::vec;

use crate::frontend::lexer::tokens::{LogicalOp, Op};
use crate::frontend::parser::{ast::*, Parser};
use environment::Environment;

use values::*;

use self::environment::{EnvironmentBuilder, EnvironmentRef};
use self::misc_functions::is_truthy;

pub struct Interpreter {
    pub stack: Vec<EvalResult>,
    pub env: EnvironmentRef,
    call_nesting: u32,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            env: EnvironmentBuilder::new().build(),
            call_nesting: 0,
        }
    }

    pub fn with_environment(env: &EnvironmentRef) -> Self {
        Self {
            stack: Vec::new(),
            env: env.clone(),
            call_nesting: 0,
        }
    }

    pub fn with_prelude() -> Self {
        let env = EnvironmentBuilder::new().with_prelude(true).build();
        Self::with_environment(&env)
    }

    pub fn set_value(&mut self, key: String, value: ValueRef) {
        self.env.borrow_mut().set(key, value);
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
}

impl AstVisitor for Interpreter {
    fn visit_program(&mut self, program: &Program) {
        let result = self.eval_block(&program.children);
        self.stack.push(result);
    }

    fn visit_block(&mut self, block: &Block) {
        let env = self.env.clone();
        self.env = EnvironmentBuilder::new().parent(&self.env).build();

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

    fn visit_pair(&mut self, pair: &Pair) {
        let left = self.eval_ast(&pair.left);
        if left.is_err() {
            self.stack.push(left);
            return;
        }
        let left = left.unwrap();

        let right = self.eval_ast(&pair.right);
        if right.is_err() {
            self.stack.push(right);
            return;
        }
        let right = right.unwrap();

        self.stack.push(Ok(new_valueref(PairValue {
            left: left.clone(),
            right: right.clone(),
        })));
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

        self.stack.push(Ok(new_valueref(VectorValue { elements })));
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

        let type_query_name = format!("{}?", &struct_def.name);
        self.env
            .borrow_mut()
            .set_builtin(&type_query_name, &Rc::new(IsStructType::new(&struct_type)));

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

    fn visit_set_bang(&mut self, set_bang: &SetBang) {
        let defining_env = Environment::get_defining_env(&self.env, &set_bang.name);
        match defining_env {
            Some(env) => {
                let value = self.eval_ast(&set_bang.value);
                if value.is_err() {
                    self.stack.push(value);
                    return;
                }

                env.borrow_mut()
                    .set(set_bang.name.clone(), value.unwrap().clone());

                self.stack.push(Ok(new_valueref(NilValue {})));
            }
            None => {
                let err = self.new_eval_error(&format!("Undefined identifier: {}", set_bang.name));
                self.stack.push(err);
            }
        }
    }

    fn visit_if(&mut self, if_expr: &IfExpression) {
        let condition = self.eval_ast(&if_expr.condition);
        if condition.is_err() {
            self.stack.push(condition);
            return;
        }
        let condition = condition.unwrap();

        let result = if is_truthy(&condition) {
            self.eval_ast(&if_expr.consequent)
        } else {
            self.eval_ast(&if_expr.alternate)
        };

        self.stack.push(result);
    }

    fn visit_lambda(&mut self, lambda: &Lambda) {
        let lambda_value = new_valueref(LambdaValue::new(
            lambda.name.clone(),
            lambda.parameters.clone(),
            lambda.variadic.clone(),
            &lambda.body,
            &self.env,
        ));
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
            ValueType::Lambda => {
                let lambda = downcast_value::<LambdaValue>(&callee).unwrap();
                if let Some(name) = &lambda.name {
                    if name == "main" {
                        let err = self.new_eval_error("Cannot call main function");
                        self.stack.push(err);
                        return;
                    } else {
                        lambda
                    }
                } else {
                    lambda
                }
            }
            ValueType::BuiltInFunction => downcast_value::<BuiltInFunctionValue>(&callee).unwrap(),
            _ => {
                let err = self.new_eval_error("Callee is not a function");
                self.stack.push(err);
                return;
            }
        };

        self.call_nesting += 1;

        let mut args = vec![];
        for arg in &call.arguments {
            let arg = self.eval_ast(arg);
            if arg.is_err() {
                self.stack.push(arg);
                self.call_nesting -= 1;
                return;
            }
            let arg = arg.unwrap();
            if arg.borrow().get_type() == ValueType::Spread {
                let spread = &borrow_value(&arg);
                let spread = downcast_value::<SpreadValue>(&spread).unwrap();
                for element in &spread.elements {
                    args.push(element.clone());
                }
                continue;
            }
            args.push(arg);
        }

        if call.is_tail_call {
            let tail_call = new_valueref(TailCallValue {
                arguments: args.clone(),
            });
            self.stack.push(Ok(tail_call));
            self.call_nesting -= 1;
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
                    self.call_nesting -= 1;
                    return;
                }
                Err(_) => {
                    self.stack.push(result);
                    self.call_nesting -= 1;
                    return;
                }
            }
        }
    }

    fn visit_spread_expr(&mut self, spread_expr: &SpreadExpr) {
        if self.call_nesting == 0 {
            let err = self.new_eval_error("Spread expression outside of function call");
            self.stack.push(err);
            return;
        }

        let spread_value = self.eval_ast(&spread_expr.expr);
        if spread_value.is_err() {
            self.stack.push(spread_value);
            return;
        }

        let spread_value = spread_value.unwrap();
        let spread_value = &borrow_value(&spread_value);
        let spread_value = downcast_value::<VectorValue>(spread_value);

        if spread_value.is_none() {
            let err = self.new_eval_error("Spread expression must be a list");
            self.stack.push(err);
            return;
        }

        let spread_value = spread_value.unwrap();

        let spread = SpreadValue {
            elements: spread_value.elements.clone(),
        };

        self.stack.push(Ok(new_valueref(spread)));
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
    fn test_eval_pair() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(1 . 2)").unwrap();
        let result = &borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Pair);
        assert_eq!(result.to_string(), "(1 . 2)");
    }

    #[test]
    fn test_eval_list() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("'(1 2 3 (4 5))").unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Vector);
        assert_eq!(result.to_string(), "(vector 1 2 3 (vector 4 5))");
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
    fn test_eval_string_head_w_unicode() {
        let mut interpreter = Interpreter::with_prelude();
        let code = r#"
            (def 🦀-str "🦀 Hello, World!")
            (head 🦀-str)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Str);
        assert_eq!(result.to_string(), "\"🦀\"");
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
        assert_eq!(result.get_type(), ValueType::Vector);
        assert_eq!(
            result.to_string(),
            "(vector (vector '+ 1 2) (vector 'a 'b))"
        );
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
    fn test_is_struct_ok() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def-struct person (name first-name))
            (def philosophus (create-person "Nietzsche" "Friedrich"))
            (person? philosophus)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#true");
    }

    #[test]
    fn test_is_struct_no_struct() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def-struct person (name first-name))
            (person? 42)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Bool);
        assert_eq!(result.to_string(), "#false");
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
    fn test_hash_length() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def options (create-hash-table))
            (hash-set! options 'action "parse")
            (hash-set! options 'input-file "code.boli")
            (hash-length options)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "2");
    }

    #[test]
    fn test_hash_keys() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def options (create-hash-table))
            (hash-set! options 'action "parse")
            (hash-set! options 'input-file "code.boli")
            (hash-remove! options 'input-file)
            (hash-keys options)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Vector);
        assert_eq!(result.to_string(), "(vector 'action)");
    }

    #[test]
    fn test_create_set() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def s (create-set))
            (set-add! s 42)
            s
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Struct);
        assert_eq!(result.to_string(), "(set 42)");
    }

    #[test]
    fn test_set_length() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def s (create-set 42 42 43))
            (set-length s)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "2");
    }

    #[test]
    fn test_set_elements() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def s (create-set))
            (set-add! s 42)
            (set-elements s)
        "#;
        let result = interpreter.eval(code).unwrap();
        let result = borrow_value(&result);
        assert_eq!(result.get_type(), ValueType::Vector);
        assert_eq!(result.to_string(), "(vector 42)");
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

    #[test]
    fn test_eval_varargs() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def (my-add numbers...)
                (+ ...numbers))
            (my-add 1 2 3 4 5)
        "#;
        let result = interpreter.eval(code).unwrap();
        assert_eq!(result.borrow().get_type(), ValueType::Int);
        assert_eq!(result.borrow().to_string(), "15");
    }

    #[test]
    fn test_eval_not() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (not (= 1 2))
        "#;
        let result = interpreter.eval(code).unwrap();
        assert_eq!(result.borrow().get_type(), ValueType::Bool);
        assert_eq!(result.borrow().to_string(), "#true");
    }

    #[test]
    fn test_eval_set_bang() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def x 42)
            (def (run)
                (set! x 43)
                42)
            (run)
            x
        "#;
        let result = interpreter.eval(code).unwrap();
        assert_eq!(result.borrow().get_type(), ValueType::Int);
        assert_eq!(result.borrow().to_string(), "43");
    }

    #[test]
    fn test_mult_arities() {
        let mut interpreter = Interpreter::new();
        let code = r#"
            (def (add x)
                (add x 1))
            (def (add x y)
                (+ x y))
            (+ (add 42) (add 42 43))
        "#;
        let result = interpreter.eval(code).unwrap();
        assert_eq!(result.borrow().get_type(), ValueType::Int);
        assert_eq!(result.borrow().to_string(), "128");
    }
}

pub mod environment;
pub mod list_functions;
pub mod number_functions;
pub mod values;

use std::cell::RefCell;
use std::rc::Rc;

use crate::frontend::lexer::tokens::{LogicalOp, Op};
use crate::frontend::parser::{ast::*, Parser};
use environment::Environment;

use values::*;

pub struct Interpreter {
    pub stack: Vec<EvalResult>,
    pub env: Rc<RefCell<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            env: Rc::new(RefCell::new(Environment::new())),
        }
    }

    pub fn with_environment(env: &Rc<RefCell<Environment>>) -> Self {
        Self {
            stack: Vec::new(),
            env: env.clone(),
        }
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

    fn visit_absolute_name(&mut self, _absolute_name: &AbsoluteName) {
        todo!()
    }

    fn visit_symbol(&mut self, _symbol: &Symbol) {
        todo!()
    }

    fn visit_quote(&mut self, _quote: &Quote) {
        todo!()
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

    fn visit_struct_def(&mut self, _struct_def: &StructDefinition) {
        todo!()
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
        let callable = Rc::new(callable);

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
}

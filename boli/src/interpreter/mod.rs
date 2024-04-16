pub mod builtins;
pub mod environment;
pub mod values;

use std::rc::Rc;

use crate::frontend::lexer::tokens::Op;
use crate::frontend::parser::{ast::*, Parser};
use environment::Environment;

use values::*;

pub struct Interpreter {
    pub stack: Vec<EvalResult>,
    pub env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            env: Environment::new(),
        }
    }

    pub fn eval(&mut self, code: &str) -> EvalResult {
        let parser = Parser::new();
        let program: Rc<dyn Ast> = Rc::new(
            parser
                .parse(code)
                .map_err(|e| InterpreterError::new(&e.message))?,
        );

        self.eval_ast(&program)
    }

    fn new_eval_error(&mut self, message: &str) -> EvalResult {
        Err(InterpreterError::new(message))
    }

    fn eval_ast(&mut self, ast: &Rc<dyn Ast>) -> EvalResult {
        ast.accept(self);
        self.stack
            .pop()
            .unwrap_or(self.new_eval_error("No value on the stack"))
    }

    fn eval_block(&mut self, children: &Vec<Rc<dyn Ast>>) -> EvalResult {
        let mut result: EvalResult = Ok(Rc::new(NilValue {}));

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
        let result = self.eval_block(&block.children);
        self.stack.push(result);
    }

    fn visit_integer(&mut self, integer: &Integer) {
        self.stack.push(Ok(Rc::new(IntValue {
            value: integer.value,
        })));
    }

    fn visit_real(&mut self, real: &Real) {
        self.stack
            .push(Ok(Rc::new(RealValue { value: real.value })));
    }

    fn visit_bool(&mut self, bool: &Bool) {
        todo!()
    }

    fn visit_str(&mut self, str: &Str) {
        todo!()
    }

    fn visit_nil(&mut self) {
        todo!()
    }

    fn visit_identifier(&mut self, identifier: &Identifier) {
        todo!()
    }

    fn visit_absolute_name(&mut self, absolute_name: &AbsoluteName) {
        todo!()
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        todo!()
    }

    fn visit_quote(&mut self, quote: &Quote) {
        todo!()
    }

    fn visit_operator(&mut self, operator: &Operator) {
        match operator.value {
            Op::Plus => {
                let add = self.env.get("+").unwrap();
                self.stack.push(Ok(add.clone()));
            }
            Op::Minus => {
                let sub = self.env.get("-").unwrap();
                self.stack.push(Ok(sub.clone()));
            }
            Op::Asterisk => {
                let mul = self.env.get("*").unwrap();
                self.stack.push(Ok(mul.clone()));
            }
            Op::Slash => {
                let div = self.env.get("/").unwrap();
                self.stack.push(Ok(div.clone()));
            }
            Op::Caret => {
                let pow = self.env.get("^").unwrap();
                self.stack.push(Ok(pow.clone()));
            }
            Op::Percent => {
                let mod_ = self.env.get("%").unwrap();
                self.stack.push(Ok(mod_.clone()));
            }
        }
    }

    fn visit_logical_operator(&mut self, operator: &LogicalOperator) {
        todo!()
    }

    fn visit_list(&mut self, list: &List) {
        todo!()
    }

    fn visit_def(&mut self, def: &Definition) {
        todo!()
    }

    fn visit_struct_def(&mut self, struct_def: &StructDefinition) {
        todo!()
    }

    fn visit_if(&mut self, if_expr: &IfExpression) {
        todo!()
    }

    fn visit_lambda(&mut self, lambda: &Lambda) {
        todo!()
    }

    fn visit_call(&mut self, call: &Call) {
        let callee = self.eval_ast(&call.callee);
        if callee.is_err() {
            self.stack.push(callee);
            return;
        }
        let callee = callee.unwrap();
        let callee_type = callee.get_type();

        let callable: &dyn Callable = match callee_type {
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

        let result = callable.call(&args);
        self.stack.push(result);
    }

    fn visit_spread_expr(&mut self, spread_expr: &SpreadExpr) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_integer() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("42").unwrap();
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_real() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("42,0").unwrap();
        assert_eq!(result.get_type(), ValueType::Real);
        assert_eq!(result.to_string(), "42,0");
    }

    #[test]
    fn test_eval_addition() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(+ 1 2 3 4,0)").unwrap();
        assert_eq!(result.get_type(), ValueType::Real);
        assert_eq!(result.to_string(), "10,0");
    }

    #[test]
    fn test_eval_subtraction() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(- 44 1 1)").unwrap();
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_multiplication() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(* 2 3 7)").unwrap();
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_division() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(/ 84,5 2)").unwrap();
        assert_eq!(result.get_type(), ValueType::Real);
        assert_eq!(result.to_string(), "42,25");
    }

    #[test]
    fn test_eval_power() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(^ 2 2 3)").unwrap();
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "256");
    }

    #[test]
    fn test_eval_modulo() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(% 85 43)").unwrap();
        assert_eq!(result.get_type(), ValueType::Int);
        assert_eq!(result.to_string(), "42");
    }
}

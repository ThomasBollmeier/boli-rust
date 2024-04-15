pub mod builtins;
pub mod environment;
pub mod values;

use std::rc::Rc;

use crate::frontend::lexer::tokens::Op;
use crate::frontend::parser::{ast::*, Parser};
use environment::Environment;
use std::error::Error;
use values::*;

pub struct Interpreter {
    pub stack: Vec<Rc<dyn Value>>,
    pub env: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            env: Environment::new(),
        }
    }

    pub fn eval(&mut self, code: &str) -> Result<Rc<dyn Value>, InterpreterError> {
        let parser = Parser::new();
        let program = parser
            .parse(code)
            .map_err(|e| InterpreterError::new(&e.message))?;

        let mut result: Rc<dyn Value> = Rc::new(NilValue {});

        for child in program.children {
            child.accept(self);
            if let Some(value) = self.stack.pop() {
                result = value;
            }
        }

        Ok(result)
    }
}

impl AstVisitor for Interpreter {
    fn visit_program(&mut self, program: &Program) {
        todo!()
    }

    fn visit_block(&mut self, block: &Block) {
        todo!()
    }

    fn visit_integer(&mut self, integer: &Integer) {
        self.stack.push(Rc::new(IntValue {
            value: integer.value,
        }));
    }

    fn visit_real(&mut self, real: &Real) {
        todo!()
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
                self.stack.push(add.clone());
            }
            _ => todo!(),
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
        call.callee.accept(self);
        let value = self.stack.pop().unwrap();
        let callee = downcast_value::<BuiltInFunction>(&value).unwrap();

        let mut args = vec![];
        for arg in &call.arguments {
            arg.accept(self);
            args.push(self.stack.pop().unwrap());
        }

        let result = callee.function.call(args);
        self.stack.push(result);
    }

    fn visit_spread_expr(&mut self, spread_expr: &SpreadExpr) {
        todo!()
    }
}

#[derive(Debug)]
pub struct InterpreterError {
    pub message: String,
}

impl InterpreterError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InterpreterError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_integer() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("42").unwrap();
        assert_eq!(result.to_string(), "42");
    }

    #[test]
    fn test_eval_addition() {
        let mut interpreter = Interpreter::new();
        let result = interpreter.eval("(+ 1 2)").unwrap();
        assert_eq!(result.to_string(), "3");
    }
}

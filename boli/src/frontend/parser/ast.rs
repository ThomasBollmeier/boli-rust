use crate::frontend::lexer::tokens::{self, Token};
use std::rc::Rc;

pub trait Ast {
    fn accept(&self, visitor: &mut dyn AstVisitor);
    fn as_any(&self) -> &dyn std::any::Any;
}

pub fn downcast_ast<T: 'static>(ast: &Rc<dyn Ast>) -> Option<&T> {
    ast.as_any().downcast_ref::<T>()
}

pub trait AstVisitor {
    fn visit_program(&mut self, program: &Program);
    fn visit_integer(&mut self, integer: &Integer);
    fn visit_real(&mut self, real: &Real);
    fn visit_bool(&mut self, bool: &Bool);
    fn visit_str(&mut self, str: &Str);
    fn visit_nil(&mut self);
    fn visit_identifier(&mut self, identifier: &Identifier);
    fn visit_symbol(&mut self, symbol: &Symbol);
    fn visit_quote(&mut self, quote: &Quote);
    fn visit_operator(&mut self, operator: &Operator);
    fn visit_logical_operator(&mut self, operator: &LogicalOperator);
    fn visit_list(&mut self, list: &List);
    fn visit_def(&mut self, def: &Definition);
    fn visit_if(&mut self, if_expr: &IfExpression);
    fn visit_lambda(&mut self, lambda: &Lambda);
    fn visit_call(&mut self, call: &Call);
}

pub struct Program {
    pub children: Vec<Rc<dyn Ast>>,
}

impl Ast for Program {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_program(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Integer {
    pub value: i64,
}

impl Ast for Integer {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_integer(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Real {
    pub value: f64,
}

impl Ast for Real {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_real(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Bool {
    pub value: bool,
}

impl Ast for Bool {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_bool(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
pub struct Str {
    pub value: String,
}

impl Ast for Str {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_str(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Nil {}

impl Ast for Nil {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_nil();
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Identifier {
    pub value: String,
}

impl Ast for Identifier {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_identifier(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Symbol {
    pub value: String,
}

impl Ast for Symbol {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_symbol(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Quote {
    pub value: Token,
}

impl Ast for Quote {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_quote(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Operator {
    pub value: tokens::Op,
}

impl Ast for Operator {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_operator(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct LogicalOperator {
    pub value: tokens::LogicalOp,
}

impl Ast for LogicalOperator {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_logical_operator(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct List {
    pub elements: Vec<Rc<dyn Ast>>,
}

impl Ast for List {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_list(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Definition {
    pub name: String,
    pub value: Rc<dyn Ast>,
}

impl Ast for Definition {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_def(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct IfExpression {
    pub condition: Rc<dyn Ast>,
    pub consequent: Rc<dyn Ast>,
    pub alternate: Rc<dyn Ast>,
}

impl Ast for IfExpression {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_if(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Lambda {
    pub parameters: Vec<String>,
    pub body: Vec<Rc<dyn Ast>>,
}

impl Ast for Lambda {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_lambda(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Call {
    pub callee: Rc<dyn Ast>,
    pub arguments: Vec<Rc<dyn Ast>>,
}

impl Ast for Call {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_call(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

use crate::frontend::lexer::tokens::{self, Token};
use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub trait Ast {
    fn accept(&self, visitor: &mut dyn AstVisitor);
    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor);
    fn as_any(&self) -> &dyn std::any::Any;
}

pub type AstRef = Rc<RefCell<dyn Ast>>;

pub fn new_astref<T: Ast + 'static>(ast: T) -> AstRef {
    Rc::new(RefCell::new(ast))
}

pub fn borrow_ast<'a>(ast: &'a AstRef) -> Ref<'a, dyn Ast> {
    ast.borrow()
}

pub fn downcast_ast<'a, T: 'static>(ast: &'a Ref<dyn Ast>) -> Option<&'a T> {
    ast.as_any().downcast_ref::<T>()
}

pub trait AstVisitor {
    fn visit_program(&mut self, program: &Program);
    fn visit_block(&mut self, block: &Block);
    fn visit_integer(&mut self, integer: &Integer);
    fn visit_real(&mut self, real: &Real);
    fn visit_bool(&mut self, bool: &Bool);
    fn visit_str(&mut self, str: &Str);
    fn visit_nil(&mut self);
    fn visit_identifier(&mut self, identifier: &Identifier);
    fn visit_absolute_name(&mut self, absolute_name: &AbsoluteName);
    fn visit_symbol(&mut self, symbol: &Symbol);
    fn visit_quote(&mut self, quote: &Quote);
    fn visit_operator(&mut self, operator: &Operator);
    fn visit_logical_operator(&mut self, operator: &LogicalOperator);
    fn visit_pair(&mut self, pair: &Pair);
    fn visit_list(&mut self, list: &List);
    fn visit_def(&mut self, def: &Definition);
    fn visit_struct_def(&mut self, struct_def: &StructDefinition);
    fn visit_if(&mut self, if_expr: &IfExpression);
    fn visit_lambda(&mut self, lambda: &Lambda);
    fn visit_call(&mut self, call: &Call);
    fn visit_spread_expr(&mut self, spread_expr: &SpreadExpr);
}

pub trait AstMutVisitor {
    fn visit_program(&mut self, program: &mut Program);
    fn visit_block(&mut self, block: &mut Block);
    fn visit_integer(&mut self, integer: &mut Integer);
    fn visit_real(&mut self, real: &mut Real);
    fn visit_bool(&mut self, bool: &mut Bool);
    fn visit_str(&mut self, str: &mut Str);
    fn visit_nil(&mut self);
    fn visit_identifier(&mut self, identifier: &mut Identifier);
    fn visit_absolute_name(&mut self, absolute_name: &mut AbsoluteName);
    fn visit_symbol(&mut self, symbol: &mut Symbol);
    fn visit_quote(&mut self, quote: &mut Quote);
    fn visit_operator(&mut self, operator: &mut Operator);
    fn visit_logical_operator(&mut self, operator: &mut LogicalOperator);
    fn visit_pair(&mut self, pair: &mut Pair);
    fn visit_list(&mut self, list: &mut List);
    fn visit_def(&mut self, def: &mut Definition);
    fn visit_struct_def(&mut self, struct_def: &mut StructDefinition);
    fn visit_if(&mut self, if_expr: &mut IfExpression);
    fn visit_lambda(&mut self, lambda: &mut Lambda);
    fn visit_call(&mut self, call: &mut Call);
    fn visit_spread_expr(&mut self, spread_expr: &mut SpreadExpr);
}

pub struct Program {
    pub children: Vec<AstRef>,
}

impl Ast for Program {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_program(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_program(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Block {
    pub children: Vec<AstRef>,
}

impl Ast for Block {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_block(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_block(self);
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_identifier(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct AbsoluteName {
    pub segments: Vec<String>,
}

impl Ast for AbsoluteName {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_absolute_name(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_absolute_name(self);
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
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

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_logical_operator(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Pair {
    pub left: AstRef,
    pub right: AstRef,
}

impl Ast for Pair {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_pair(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_pair(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct List {
    pub elements: Vec<AstRef>,
}

impl Ast for List {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_list(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_list(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Definition {
    pub name: String,
    pub value: AstRef,
}

impl Ast for Definition {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_def(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_def(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct StructDefinition {
    pub name: String,
    pub fields: Vec<String>,
}

impl Ast for StructDefinition {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_struct_def(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_struct_def(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct IfExpression {
    pub condition: AstRef,
    pub consequent: AstRef,
    pub alternate: AstRef,
}

impl Ast for IfExpression {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_if(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_if(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Lambda {
    pub name: Option<String>,
    pub parameters: Vec<String>,
    pub variadic: Option<String>,
    pub body: AstRef,
}

impl Ast for Lambda {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_lambda(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_lambda(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Call {
    pub callee: AstRef,
    pub arguments: Vec<AstRef>,
    pub is_tail_call: bool,
}

impl Ast for Call {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_call(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_call(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct SpreadExpr {
    pub expr: AstRef,
}

impl Ast for SpreadExpr {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_spread_expr(self);
    }

    fn accept_mut(&mut self, visitor: &mut dyn AstMutVisitor) {
        visitor.visit_spread_expr(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

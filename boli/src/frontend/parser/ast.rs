pub trait Ast {
    fn accept(&self, visitor: &mut dyn AstVisitor);
    fn as_any(&self) -> &dyn std::any::Any;
}

pub fn downcast_ast<T: 'static>(ast: &Box<dyn Ast>) -> Option<&T> {
    ast.as_any().downcast_ref::<T>()
}

pub trait AstVisitor {
    fn visit_program(&mut self, program: &Program);
    fn visit_integer(&mut self, integer: &Integer);
    fn visit_real(&mut self, real: &Real);
    fn visit_bool(&mut self, bool: &Bool);
    fn visit_str(&mut self, str: &Str);
    fn visit_def(&mut self, def: &Definition);
    fn visit_if(&mut self, if_expr: &IfExpression);
}

pub struct Program {
    pub children: Vec<Box<dyn Ast>>,
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

pub struct Definition {
    pub name: String,
    pub value: Box<dyn Ast>,
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
    pub condition: Box<dyn Ast>,
    pub consequent: Box<dyn Ast>,
    pub alternate: Box<dyn Ast>,
}

impl Ast for IfExpression {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        visitor.visit_if(self);
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

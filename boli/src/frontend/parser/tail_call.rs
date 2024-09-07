use super::ast::*;

pub struct TailCallFinder {}

impl TailCallFinder {
    pub fn new() -> Self {
        TailCallFinder {}
    }

    pub fn mark_tail_calls(&mut self, program: &mut Program) {
        program.accept_mut(self);
    }
}

impl AstMutVisitor for TailCallFinder {
    fn visit_program(&mut self, program: &mut Program) {
        program.children.iter_mut().for_each(|child| {
            child.borrow_mut().accept_mut(self);
        });
    }

    fn visit_block(&mut self, block: &mut Block) {
        block.children.iter_mut().for_each(|child| {
            child.borrow_mut().accept_mut(self);
        });
    }

    fn visit_integer(&mut self, _integer: &mut Integer) {}

    fn visit_rational(&mut self, _rational: &mut Rational) {}

    fn visit_real(&mut self, _real: &mut Real) {}

    fn visit_bool(&mut self, _bool: &mut Bool) {}

    fn visit_str(&mut self, _str: &mut Str) {}

    fn visit_nil(&mut self) {}

    fn visit_identifier(&mut self, _identifier: &mut Identifier) {}

    fn visit_absolute_name(&mut self, _absolute_name: &mut AbsoluteName) {}

    fn visit_symbol(&mut self, _symbol: &mut Symbol) {}

    fn visit_quote(&mut self, _quote: &mut Quote) {}

    fn visit_operator(&mut self, _operator: &mut Operator) {}

    fn visit_logical_operator(&mut self, _operator: &mut LogicalOperator) {}

    fn visit_pair(&mut self, pair: &mut Pair) {
        pair.left.borrow_mut().accept_mut(self);
        pair.right.borrow_mut().accept_mut(self);
    }

    fn visit_list(&mut self, list: &mut List) {
        list.elements.iter_mut().for_each(|element| {
            element.borrow_mut().accept_mut(self);
        });
    }

    fn visit_def(&mut self, def: &mut Definition) {
        def.value.borrow_mut().accept_mut(self);
    }

    fn visit_struct_def(&mut self, _struct_def: &mut StructDefinition) {}

    fn visit_set_bang(&mut self, set_bang: &mut SetBang) {
        set_bang.value.borrow_mut().accept_mut(self);
    }

    fn visit_if(&mut self, if_expr: &mut IfExpression) {
        if_expr.condition.borrow_mut().accept_mut(self);
        if_expr.consequent.borrow_mut().accept_mut(self);
        if_expr.alternate.borrow_mut().accept_mut(self);
    }

    fn visit_lambda(&mut self, lambda: &mut Lambda) {
        TailCallSetter::mark_tail_calls(lambda);
        lambda.body.borrow_mut().accept_mut(self);
    }

    fn visit_call(&mut self, call: &mut Call) {
        call.callee.borrow_mut().accept_mut(self);
    }

    fn visit_spread_expr(&mut self, spread_expr: &mut SpreadExpr) {
        spread_expr.expr.borrow_mut().accept_mut(self);
    }
}

struct TailCallSetter {
    current_lambda: String,
}

impl TailCallSetter {
    fn mark_tail_calls(lambda: &mut Lambda) {
        if let Some(name) = &lambda.name {
            let mut setter = TailCallSetter {
                current_lambda: name.clone(),
            };
            lambda.body.borrow_mut().accept_mut(&mut setter);
        }
    }
}

impl AstMutVisitor for TailCallSetter {
    fn visit_program(&mut self, _program: &mut Program) {}

    fn visit_block(&mut self, block: &mut Block) {
        if let Some(last_child) = block.children.last_mut() {
            last_child.borrow_mut().accept_mut(self);
        }
    }

    fn visit_integer(&mut self, _integer: &mut Integer) {}

    fn visit_rational(&mut self, _rational: &mut Rational) {}

    fn visit_real(&mut self, _real: &mut Real) {}

    fn visit_bool(&mut self, _bool: &mut Bool) {}

    fn visit_str(&mut self, _str: &mut Str) {}

    fn visit_nil(&mut self) {}

    fn visit_identifier(&mut self, _identifier: &mut Identifier) {}

    fn visit_absolute_name(&mut self, _absolute_name: &mut AbsoluteName) {}

    fn visit_symbol(&mut self, _symbol: &mut Symbol) {}

    fn visit_quote(&mut self, _quote: &mut Quote) {}

    fn visit_operator(&mut self, _operator: &mut Operator) {}

    fn visit_logical_operator(&mut self, _operator: &mut LogicalOperator) {}

    fn visit_pair(&mut self, _pair: &mut Pair) {}

    fn visit_list(&mut self, _list: &mut List) {}

    fn visit_def(&mut self, _def: &mut Definition) {}

    fn visit_struct_def(&mut self, _struct_def: &mut StructDefinition) {}

    fn visit_set_bang(&mut self, _set_bang: &mut SetBang) {}

    fn visit_if(&mut self, if_expr: &mut IfExpression) {
        if_expr.consequent.borrow_mut().accept_mut(self);
        if_expr.alternate.borrow_mut().accept_mut(self);
    }

    fn visit_lambda(&mut self, lambda: &mut Lambda) {
        lambda.body.borrow_mut().accept_mut(self);
    }

    fn visit_call(&mut self, call: &mut Call) {
        let callee = &borrow_ast(&call.callee);
        let callee = downcast_ast::<Identifier>(callee);

        if let Some(ident) = callee {
            if ident.value == self.current_lambda {
                call.is_tail_call = true;
            }
        }
    }

    fn visit_spread_expr(&mut self, _spread_expr: &mut SpreadExpr) {}
}

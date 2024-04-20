use super::ast::*;
use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

#[derive(Debug, PartialEq, Clone)]
pub enum JsonData {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonData>),
    Object(HashMap<String, JsonData>, Vec<String>),
}

impl JsonData {
    fn pretty_print(&self) -> String {
        let mut result = String::new();
        self.pretty_print_internal(&mut result, 0);
        result
    }

    fn pretty_print_internal(&self, result: &mut String, indent: usize) {
        match self {
            JsonData::Null => result.push_str("null"),
            JsonData::Bool(value) => result.push_str(&format!("{}", value)),
            JsonData::Number(value) => result.push_str(&format!("{}", value)),
            JsonData::String(value) => result.push_str(&format!("\"{}\"", value)),
            JsonData::Array(elements) => {
                result.push(char::from('['));
                if !elements.is_empty() {
                    result.push(char::from('\n'));
                    let last = elements.len() - 1;
                    for (i, element) in elements.iter().enumerate() {
                        result.push_str(&" ".repeat(indent + 2));
                        element.pretty_print_internal(result, indent + 2);
                        if i != last {
                            result.push(char::from(','));
                        }
                        result.push(char::from('\n'));
                    }
                    result.push_str(&" ".repeat(indent));
                }
                result.push(char::from(']'));
            }
            JsonData::Object(data, fields) => {
                result.push(char::from('{'));
                if !fields.is_empty() {
                    result.push(char::from('\n'));
                    let last = fields.len() - 1;
                    for (i, field) in fields.iter().enumerate() {
                        result.push_str(&" ".repeat(indent + 2));
                        result.push_str(&format!("\"{}\": ", field));
                        data.get(field)
                            .unwrap()
                            .pretty_print_internal(result, indent + 2);
                        if i != last {
                            result.push(char::from(','));
                        }
                        result.push(char::from('\n'));
                    }
                    result.push_str(&" ".repeat(indent));
                }
                result.push(char::from('}'));
            }
        }
    }
}

impl Display for JsonData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.pretty_print())
    }
}

impl From<Program> for JsonData {
    fn from(program: Program) -> Self {
        let mut visitor = AstToJsonVisitor::new();
        visitor.to_json(&program)
    }
}

pub struct AstToJsonVisitor {
    stack: Vec<JsonData>,
}

impl AstToJsonVisitor {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn to_json(&mut self, ast: &dyn Ast) -> JsonData {
        ast.accept(self);
        self.stack.pop().unwrap()
    }

    fn new_object_content() -> (HashMap<String, JsonData>, Vec<String>) {
        (HashMap::new(), Vec::new())
    }

    fn add_field(
        name: &str,
        value: JsonData,
        data: &mut HashMap<String, JsonData>,
        fields: &mut Vec<String>,
    ) {
        data.insert(name.to_string(), value);
        fields.push(name.to_string());
    }
}

impl AstVisitor for AstToJsonVisitor {
    fn visit_program(&mut self, program: &Program) {
        let (mut data, mut fields) = Self::new_object_content();
        let mut children: Vec<JsonData> = Vec::new();

        Self::add_field(
            "type",
            JsonData::String("Program".to_string()),
            &mut data,
            &mut fields,
        );

        for child in &program.children {
            child.borrow().accept(self);
            children.push(self.stack.pop().unwrap());
        }

        Self::add_field(
            "children",
            JsonData::Array(children),
            &mut data,
            &mut fields,
        );

        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_block(&mut self, block: &Block) {
        let (mut data, mut fields) = Self::new_object_content();
        let mut children: Vec<JsonData> = Vec::new();

        Self::add_field(
            "type",
            JsonData::String("Block".to_string()),
            &mut data,
            &mut fields,
        );

        for child in &block.children {
            child.borrow().accept(self);
            children.push(self.stack.pop().unwrap());
        }

        Self::add_field(
            "children",
            JsonData::Array(children),
            &mut data,
            &mut fields,
        );

        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_integer(&mut self, integer: &Integer) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Integer".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field(
            "value",
            JsonData::Number(integer.value as f64),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_real(&mut self, real: &Real) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Real".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field(
            "value",
            JsonData::Number(real.value),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_bool(&mut self, bool: &Bool) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Boolean".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field("value", JsonData::Bool(bool.value), &mut data, &mut fields);
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_str(&mut self, str: &Str) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("String".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field(
            "value",
            JsonData::String(str.value.clone()),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_nil(&mut self) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Nil".to_string()),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_identifier(&mut self, identifier: &Identifier) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Identifier".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field(
            "value",
            JsonData::String(identifier.value.clone()),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_absolute_name(&mut self, absolute_name: &AbsoluteName) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("AbsoluteName".to_string()),
            &mut data,
            &mut fields,
        );

        let mut segments: Vec<JsonData> = Vec::new();
        for segment in &absolute_name.segments {
            segments.push(JsonData::String(segment.clone()));
        }

        Self::add_field(
            "segments",
            JsonData::Array(segments),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_symbol(&mut self, symbol: &Symbol) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Symbol".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field(
            "value",
            JsonData::String(symbol.value.clone()),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_quote(&mut self, quote: &Quote) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Quote".to_string()),
            &mut data,
            &mut fields,
        );

        Self::add_field(
            "value",
            JsonData::String(quote.value.to_string()),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_operator(&mut self, operator: &Operator) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Operator".to_string()),
            &mut data,
            &mut fields,
        );

        Self::add_field(
            "value",
            JsonData::String(format!("{:?}", operator.value)),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_logical_operator(&mut self, operator: &LogicalOperator) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("LogicalOperator".to_string()),
            &mut data,
            &mut fields,
        );

        Self::add_field(
            "value",
            JsonData::String(format!("{:?}", operator.value)),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_list(&mut self, list: &List) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("List".to_string()),
            &mut data,
            &mut fields,
        );

        let mut elements: Vec<JsonData> = Vec::new();
        for element in &list.elements {
            element.borrow().accept(self);
            elements.push(self.stack.pop().unwrap());
        }

        Self::add_field(
            "elements",
            JsonData::Array(elements),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_def(&mut self, def: &Definition) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Definition".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field(
            "name",
            JsonData::String(def.name.clone()),
            &mut data,
            &mut fields,
        );

        def.value.borrow().accept(self);
        Self::add_field("value", self.stack.pop().unwrap(), &mut data, &mut fields);
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_struct_def(&mut self, struct_def: &StructDefinition) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("StructDefinition".to_string()),
            &mut data,
            &mut fields,
        );
        Self::add_field(
            "name",
            JsonData::String(struct_def.name.clone()),
            &mut data,
            &mut fields,
        );

        let mut struct_fields: Vec<JsonData> = Vec::new();
        for field in &struct_def.fields {
            struct_fields.push(JsonData::String(field.clone()));
        }

        Self::add_field(
            "fields",
            JsonData::Array(struct_fields),
            &mut data,
            &mut fields,
        );
        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_if(&mut self, if_expr: &IfExpression) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("IfExpression".to_string()),
            &mut data,
            &mut fields,
        );

        if_expr.condition.borrow().accept(self);
        Self::add_field(
            "condition",
            self.stack.pop().unwrap(),
            &mut data,
            &mut fields,
        );

        if_expr.consequent.borrow().accept(self);
        Self::add_field(
            "consequent",
            self.stack.pop().unwrap(),
            &mut data,
            &mut fields,
        );

        if_expr.alternate.borrow().accept(self);
        Self::add_field(
            "alternate",
            self.stack.pop().unwrap(),
            &mut data,
            &mut fields,
        );

        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_lambda(&mut self, lambda: &Lambda) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Lambda".to_string()),
            &mut data,
            &mut fields,
        );

        Self::add_field(
            "name",
            match &lambda.name {
                Some(name) => JsonData::String(name.clone()),
                None => JsonData::Null,
            },
            &mut data,
            &mut fields,
        );

        let mut parameters: Vec<JsonData> = Vec::new();
        for parameter in &lambda.parameters {
            parameters.push(JsonData::String(parameter.clone()));
        }

        Self::add_field(
            "parameters",
            JsonData::Array(parameters),
            &mut data,
            &mut fields,
        );

        Self::add_field(
            "variadic",
            match &lambda.variadic {
                Some(variadic) => JsonData::String(variadic.clone()),
                None => JsonData::Null,
            },
            &mut data,
            &mut fields,
        );

        lambda.body.borrow().accept(self);

        Self::add_field("body", self.stack.pop().unwrap(), &mut data, &mut fields);

        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_call(&mut self, call: &Call) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("Call".to_string()),
            &mut data,
            &mut fields,
        );

        Self::add_field(
            "isTailCall",
            JsonData::Bool(call.is_tail_call),
            &mut data,
            &mut fields,
        );

        call.callee.borrow().accept(self);
        Self::add_field("callee", self.stack.pop().unwrap(), &mut data, &mut fields);

        let mut arguments: Vec<JsonData> = Vec::new();
        for argument in &call.arguments {
            argument.borrow().accept(self);
            arguments.push(self.stack.pop().unwrap());
        }

        Self::add_field(
            "arguments",
            JsonData::Array(arguments),
            &mut data,
            &mut fields,
        );

        self.stack.push(JsonData::Object(data, fields));
    }

    fn visit_spread_expr(&mut self, spread_expr: &SpreadExpr) {
        let (mut data, mut fields) = Self::new_object_content();
        Self::add_field(
            "type",
            JsonData::String("SpreadExpr".to_string()),
            &mut data,
            &mut fields,
        );

        spread_expr.expr.borrow().accept(self);
        Self::add_field("expr", self.stack.pop().unwrap(), &mut data, &mut fields);

        self.stack.push(JsonData::Object(data, fields));
    }
}

#[cfg(test)]
mod tests {

    use super::super::ast::*;
    use super::*;

    #[test]
    fn test_integer() {
        let integer = Integer { value: 42 };
        let mut visitor = AstToJsonVisitor::new();
        let json = visitor.to_json(&integer);

        assert_eq!(
            json,
            JsonData::Object(
                vec![
                    ("type".to_string(), JsonData::String("Integer".to_string())),
                    ("value".to_string(), JsonData::Number(42.0)),
                ]
                .into_iter()
                .collect(),
                vec!["type".to_string(), "value".to_string()]
            )
        );
    }

    #[test]
    fn test_program() {
        let integer = Integer { value: 42 };
        let program = Program {
            children: vec![new_astref(integer)],
        };
        let mut visitor = AstToJsonVisitor::new();
        let json = visitor.to_json(&program);

        assert_eq!(
            json,
            JsonData::Object(
                vec![
                    ("type".to_string(), JsonData::String("Program".to_string())),
                    (
                        "children".to_string(),
                        JsonData::Array(vec![JsonData::Object(
                            vec![
                                ("type".to_string(), JsonData::String("Integer".to_string())),
                                ("value".to_string(), JsonData::Number(42.0)),
                            ]
                            .into_iter()
                            .collect(),
                            vec!["type".to_string(), "value".to_string()]
                        )])
                    ),
                ]
                .into_iter()
                .collect(),
                vec!["type".to_string(), "children".to_string()]
            )
        );
    }
}

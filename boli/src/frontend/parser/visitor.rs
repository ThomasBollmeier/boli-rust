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

impl Display for JsonData {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            JsonData::Null => write!(f, "null"),
            JsonData::Bool(value) => write!(f, "{}", value),
            JsonData::Number(value) => write!(f, "{}", value),
            JsonData::String(value) => write!(f, "\"{}\"", value),
            JsonData::Array(elements) => {
                let mut value = String::from("[");
                let mut first = true;

                for element in elements {
                    if !first {
                        value.push(char::from(','));
                    } else {
                        first = false;
                    }
                    value.push_str(&format!("{}", element));
                }
                value.push(char::from(']'));
                write!(f, "{}", value)
            }
            JsonData::Object(data, fields) => {
                let mut value = String::from("{");
                let mut first = true;

                for field in fields {
                    let (key, val) = data.get_key_value(field).unwrap();
                    if !first {
                        value.push(char::from(','));
                    } else {
                        first = false;
                    }
                    value.push_str(&format!("\"{}\": {}", key, val));
                }
                value.push(char::from('}'));
                write!(f, "{}", value)
            }
        }
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
            child.accept(self);
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

        def.value.accept(self);
        Self::add_field("value", self.stack.pop().unwrap(), &mut data, &mut fields);
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

        if_expr.condition.accept(self);
        Self::add_field(
            "condition",
            self.stack.pop().unwrap(),
            &mut data,
            &mut fields,
        );

        if_expr.consequent.accept(self);
        Self::add_field(
            "consequent",
            self.stack.pop().unwrap(),
            &mut data,
            &mut fields,
        );

        if_expr.alternate.accept(self);
        Self::add_field(
            "alternate",
            self.stack.pop().unwrap(),
            &mut data,
            &mut fields,
        );

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
            children: vec![Box::new(integer)],
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

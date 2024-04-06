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
    Object(HashMap<String, JsonData>),
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
            JsonData::Object(data) => {
                let mut value = String::from("{");
                let mut first = true;

                for (key, val) in data {
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

    pub fn to_json(&mut self, program: &Program) -> JsonData {
        self.visit_program(program);
        self.stack.pop().unwrap()
    }
}

impl AstVisitor for AstToJsonVisitor {
    fn visit_program(&mut self, program: &Program) {
        let mut data = HashMap::new();
        let mut children: Vec<JsonData> = Vec::new();

        data.insert("type".to_string(), JsonData::String("Program".to_string()));

        for child in &program.children {
            child.accept(self);
            children.push(self.stack.pop().unwrap());
        }

        data.insert("children".to_string(), JsonData::Array(children));

        self.stack.push(JsonData::Object(data));
    }

    fn visit_integer(&mut self, integer: &Integer) {
        let mut data = HashMap::new();
        data.insert("type".to_string(), JsonData::String("Integer".to_string()));
        data.insert("value".to_string(), JsonData::Number(integer.value as f64));
        self.stack.push(JsonData::Object(data));
    }

    fn visit_real(&mut self, real: &Real) {
        let mut data = HashMap::new();
        data.insert("type".to_string(), JsonData::String("Real".to_string()));
        data.insert("value".to_string(), JsonData::Number(real.value));
        self.stack.push(JsonData::Object(data));
    }

    fn visit_bool(&mut self, bool: &Bool) {
        let mut data = HashMap::new();
        data.insert("type".to_string(), JsonData::String("Boolean".to_string()));
        data.insert("value".to_string(), JsonData::Bool(bool.value));
        self.stack.push(JsonData::Object(data));
    }

    fn visit_str(&mut self, str: &Str) {
        let mut data = HashMap::new();
        data.insert("type".to_string(), JsonData::String("String".to_string()));
        data.insert("value".to_string(), JsonData::String(str.value.clone()));
        self.stack.push(JsonData::Object(data));
    }

    fn visit_def(&mut self, def: &Definition) {
        let mut data = HashMap::new();
        data.insert(
            "type".to_string(),
            JsonData::String("Definition".to_string()),
        );
        data.insert("name".to_string(), JsonData::String(def.name.clone()));

        def.value.accept(self);
        data.insert("value".to_string(), self.stack.pop().unwrap());

        self.stack.push(JsonData::Object(data));
    }

    fn visit_if(&mut self, if_expr: &IfExpression) {
        let mut data = HashMap::new();
        data.insert(
            "type".to_string(),
            JsonData::String("IfExpression".to_string()),
        );

        if_expr.condition.accept(self);
        data.insert("condition".to_string(), self.stack.pop().unwrap());

        if_expr.consequent.accept(self);
        data.insert("consequent".to_string(), self.stack.pop().unwrap());

        if_expr.alternate.accept(self);
        data.insert("alternate".to_string(), self.stack.pop().unwrap());

        self.stack.push(JsonData::Object(data));
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::parser::Parser;

    #[test]
    fn test_json_from_ast() {
        use super::*;

        let code = r#"
            42 
            3,14 
            #true 
            "Hello, World!" 
            (def answer 42)
            (if #true 42 23)
        "#;
        let parser = Parser::new();
        let program = parser.parse(code).unwrap();

        let actual = JsonData::from(program);

        let expected = JsonData::Object({
            let mut data = HashMap::new();
            data.insert("type".to_string(), JsonData::String("Program".to_string()));
            data.insert(
                "children".to_string(),
                JsonData::Array(vec![
                    JsonData::Object({
                        let mut data = HashMap::new();
                        data.insert("type".to_string(), JsonData::String("Integer".to_string()));
                        data.insert("value".to_string(), JsonData::Number(42.0));
                        data
                    }),
                    JsonData::Object({
                        let mut data = HashMap::new();
                        data.insert("type".to_string(), JsonData::String("Real".to_string()));
                        data.insert("value".to_string(), JsonData::Number(3.14));
                        data
                    }),
                    JsonData::Object({
                        let mut data = HashMap::new();
                        data.insert("type".to_string(), JsonData::String("Boolean".to_string()));
                        data.insert("value".to_string(), JsonData::Bool(true));
                        data
                    }),
                    JsonData::Object({
                        let mut data = HashMap::new();
                        data.insert("type".to_string(), JsonData::String("String".to_string()));
                        data.insert(
                            "value".to_string(),
                            JsonData::String("Hello, World!".to_string()),
                        );
                        data
                    }),
                    JsonData::Object({
                        let mut data = HashMap::new();
                        data.insert(
                            "type".to_string(),
                            JsonData::String("Definition".to_string()),
                        );
                        data.insert("name".to_string(), JsonData::String("answer".to_string()));
                        data.insert(
                            "value".to_string(),
                            JsonData::Object({
                                let mut data = HashMap::new();
                                data.insert(
                                    "type".to_string(),
                                    JsonData::String("Integer".to_string()),
                                );
                                data.insert("value".to_string(), JsonData::Number(42.0));
                                data
                            }),
                        );
                        data
                    }),
                    JsonData::Object({
                        let mut data = HashMap::new();
                        data.insert(
                            "type".to_string(),
                            JsonData::String("IfExpression".to_string()),
                        );
                        data.insert(
                            "condition".to_string(),
                            JsonData::Object({
                                let mut data = HashMap::new();
                                data.insert(
                                    "type".to_string(),
                                    JsonData::String("Boolean".to_string()),
                                );
                                data.insert("value".to_string(), JsonData::Bool(true));
                                data
                            }),
                        );
                        data.insert(
                            "consequent".to_string(),
                            JsonData::Object({
                                let mut data = HashMap::new();
                                data.insert(
                                    "type".to_string(),
                                    JsonData::String("Integer".to_string()),
                                );
                                data.insert("value".to_string(), JsonData::Number(42.0));
                                data
                            }),
                        );
                        data.insert(
                            "alternate".to_string(),
                            JsonData::Object({
                                let mut data = HashMap::new();
                                data.insert(
                                    "type".to_string(),
                                    JsonData::String("Integer".to_string()),
                                );
                                data.insert("value".to_string(), JsonData::Number(23.0));
                                data
                            }),
                        );
                        data
                    }),
                ]),
            );
            data
        });

        assert_eq!(actual, expected);
    }
}

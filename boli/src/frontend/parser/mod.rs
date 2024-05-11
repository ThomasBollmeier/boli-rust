use core::str;
use std::error::Error;
use std::vec;

use super::lexer::stream::BufferedStream;
use super::lexer::tokens::{Token, TokenType, TokenType::*};
use super::lexer::Lexer;

pub mod ast;
pub mod json_visitor;
pub mod tail_call;

use ast::{new_astref, AstRef};

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, code: &str) -> Result<ast::Program, ParseError> {
        let mut stream = BufferedStream::new(Box::new(Lexer::new(code)));
        let mut result = self.program(&mut stream);

        if let Ok(program) = &mut result {
            tail_call::TailCallFinder::new().mark_tail_calls(program);
        }

        result
    }

    fn next_token(
        stream: &mut BufferedStream<Token>,
        expected_types: &Vec<&TokenType>,
    ) -> Result<Token, ParseError> {
        let token = stream
            .next()
            .ok_or(ParseError::new("Unexpected end of input"))?;
        for expected_type in expected_types {
            if token.token_type == **expected_type {
                return Ok(token);
            }
        }
        if !expected_types.is_empty() {
            return Err(ParseError::with_token("Unexpected token", token));
        }
        Ok(token)
    }

    fn peek_token(
        stream: &mut BufferedStream<Token>,
        expected_types: &Vec<&TokenType>,
    ) -> Option<Token> {
        let token = stream.peek()?;
        for expected_type in expected_types {
            if token.token_type == **expected_type {
                return Some(token);
            }
        }
        if !expected_types.is_empty() {
            return None;
        }
        Some(token)
    }

    fn program(&self, stream: &mut BufferedStream<Token>) -> Result<ast::Program, ParseError> {
        let mut children = Vec::new();
        while let Some(_) = Self::peek_token(stream, &vec![]) {
            children.push(self.expression(stream, true)?);
        }

        let mut program = ast::Program { children };
        tail_call::TailCallFinder::new().mark_tail_calls(&mut program);

        Ok(program)
    }

    fn expression(
        &self,
        stream: &mut BufferedStream<Token>,
        define_allowed: bool,
    ) -> Result<AstRef, ParseError> {
        let token = Self::next_token(stream, &vec![])?;

        match token.token_type {
            Integer => Ok(new_astref(ast::Integer {
                value: token.get_int_value().unwrap(),
            })),
            Real => Ok(new_astref(ast::Real {
                value: token.get_real_value().unwrap(),
            })),
            Bool => Ok(new_astref(ast::Bool {
                value: token.get_bool_value().unwrap(),
            })),
            Str => Ok(new_astref(ast::Str {
                value: token.get_string_value().unwrap(),
            })),
            Nil => Ok(new_astref(ast::Nil {})),
            Identifier => Ok(new_astref(ast::Identifier {
                value: token.get_string_value().unwrap(),
            })),
            AbsoluteName => self.absolute_name(&token),
            Symbol => Ok(new_astref(ast::Symbol {
                value: token.get_string_value().unwrap(),
            })),
            Dot3 => Ok(new_astref(ast::SpreadExpr {
                expr: self.expression(stream, false)?,
            })),
            Operator(op) => Ok(new_astref(ast::Operator { value: op.clone() })),
            LogicalOperator(op) => Ok(new_astref(ast::LogicalOperator { value: op.clone() })),
            LeftParen | LeftBrace | LeftBracket => self.symbolic_expression(
                stream,
                Self::closing_token_type(&token.token_type),
                define_allowed,
            ),
            QuoteParen | QuoteBrace | QuoteBracket => {
                self.quoted_expression(stream, Self::closing_token_type(&token.token_type))
            }
            _ => Err(ParseError::with_token("Unexpected token", token)),
        }
    }

    fn absolute_name(&self, token: &Token) -> Result<AstRef, ParseError> {
        let value = token
            .get_string_value()
            .ok_or(ParseError::new("Invalid absolute name"))?;
        let segments = value
            .split("::")
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        Ok(new_astref(ast::AbsoluteName { segments }))
    }

    fn closing_token_type(opening_token_type: &TokenType) -> TokenType {
        match opening_token_type {
            LeftParen | QuoteParen => RightParen,
            LeftBrace | QuoteBrace => RightBrace,
            LeftBracket | QuoteBracket => RightBracket,
            _ => unreachable!(),
        }
    }

    fn quoted_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let mut elements = Vec::new();
        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            let element = self.get_quoted_element(stream)?;
            elements.push(element);
        }
        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token
        Ok(new_astref(ast::List { elements }))
    }

    fn get_quoted_element(&self, stream: &mut BufferedStream<Token>) -> Result<AstRef, ParseError> {
        let token = Self::next_token(stream, &vec![])?;
        match token.token_type {
            Integer => Ok(new_astref(ast::Integer {
                value: token.get_int_value().unwrap(),
            })),
            Real => Ok(new_astref(ast::Real {
                value: token.get_real_value().unwrap(),
            })),
            Bool => Ok(new_astref(ast::Bool {
                value: token.get_bool_value().unwrap(),
            })),
            Str => Ok(new_astref(ast::Str {
                value: token.get_string_value().unwrap(),
            })),
            Symbol => Ok(new_astref(ast::Symbol {
                value: token.get_string_value().unwrap(),
            })),
            LeftParen | LeftBrace | LeftBracket => {
                self.quoted_expression(stream, Self::closing_token_type(&token.token_type))
            }
            QuoteParen | QuoteBrace | QuoteBracket => {
                Err(ParseError::new("Quotation nesting not allowed"))
            }
            _ => Ok(new_astref(ast::Quote {
                value: token.clone(),
            })),
        }
    }

    fn symbolic_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
        define_allowed: bool,
    ) -> Result<AstRef, ParseError> {
        let token = Self::next_token(stream, &vec![])?;

        match token.token_type {
            Def => {
                if define_allowed {
                    self.definition(stream, end_token_type)
                } else {
                    Err(ParseError::new("Definition not allowed here"))
                }
            }
            DefStruct => {
                if define_allowed {
                    self.struct_definition(stream, end_token_type)
                } else {
                    Err(ParseError::new("Definition not allowed here"))
                }
            }
            If => self.if_expression(stream, end_token_type),
            Cond => self.cond_expression(stream, end_token_type),
            Conjunction => self.conjunction(stream, end_token_type),
            Disjunction => self.disjunction(stream, end_token_type),
            Lambda => self.lambda(stream, end_token_type),
            Block => self.block(stream, &end_token_type),
            Let => self.let_expression(stream, end_token_type),
            _ => {
                stream.push_back(token);
                self.call(stream, end_token_type)
            }
        }
    }

    fn block(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: &TokenType,
    ) -> Result<AstRef, ParseError> {
        let mut children = Vec::new();
        while Self::peek_token(stream, &vec![end_token_type]).is_none() {
            children.push(self.expression(stream, true)?);
        }

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(new_astref(ast::Block { children }))
    }

    fn let_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let opening_def = Self::next_token(stream, &vec![&LeftParen, &LeftBrace, &LeftBracket])?;
        let closing_def_type = Self::closing_token_type(&opening_def.token_type);

        let mut children = Vec::new();
        while Self::peek_token(stream, &vec![&closing_def_type]).is_none() {
            children.push(self.let_definition(stream)?);
        }

        Self::next_token(stream, &vec![&closing_def_type])?; // consume closing token

        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            children.push(self.expression(stream, true)?);
        }

        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token

        Ok(new_astref(ast::Block { children }))
    }

    fn let_definition(&self, stream: &mut BufferedStream<Token>) -> Result<AstRef, ParseError> {
        let opening = Self::next_token(stream, &vec![&LeftParen, &LeftBrace, &LeftBracket])?;
        let closing_type = Self::closing_token_type(&opening.token_type);

        let name_token = Self::next_token(stream, &vec![&Identifier])?;
        let name = name_token.get_string_value().unwrap();

        let value = self.expression(stream, false)?;

        Self::next_token(stream, &vec![&closing_type])?; // consume closing token

        Ok(new_astref(ast::Definition { name, value }))
    }

    fn lambda(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let opening_token = Self::next_token(stream, &vec![&LeftParen, &LeftBrace, &LeftBracket])?;
        let closing_token_type = Self::closing_token_type(&opening_token.token_type);
        let mut variadic: Option<String> = None;

        let mut parameters = Vec::new();
        while Self::peek_token(stream, &vec![&Identifier]).is_some() {
            let token = Self::next_token(stream, &vec![&Identifier])?;
            if Self::peek_token(stream, &vec![&Dot3]).is_none() {
                parameters.push(token.get_string_value().unwrap());
            } else {
                Self::next_token(stream, &vec![&Dot3])?; // consume variadic token
                variadic = Some(token.get_string_value().unwrap());
                break;
            }
        }

        Self::next_token(stream, &vec![&closing_token_type])?;

        let body = self.block(stream, &end_token_type)?;

        Ok(new_astref(ast::Lambda {
            name: None,
            parameters,
            variadic,
            body,
        }))
    }

    fn call(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let callee = self.expression(stream, false)?;

        // Check if it's a pair
        if Self::peek_token(stream, &vec![&TokenType::Dot]).is_some() {
            let left = callee;
            Self::next_token(stream, &vec![&TokenType::Dot])?; // consume dot
            let right = self.expression(stream, false)?;
            Self::next_token(stream, &vec![&end_token_type])?; // consume closing token
            return Ok(new_astref(ast::Pair { left, right }));
        }

        let mut arguments = Vec::new();

        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            arguments.push(self.expression(stream, false)?);
        }

        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token

        Ok(new_astref(ast::Call {
            callee,
            arguments,
            is_tail_call: false,
        }))
    }

    fn disjunction(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let mut children = Vec::new();

        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            children.push(self.expression(stream, false)?);
        }

        if children.is_empty() {
            return Err(ParseError::new("At least one expression required"));
        }

        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token

        Ok(self.create_if_expr_from_disjunction(&mut children))
    }

    fn create_if_expr_from_disjunction(&self, elements: &mut Vec<AstRef>) -> AstRef {
        if elements.is_empty() {
            return new_astref(ast::Bool { value: false });
        }

        let condition = elements.remove(0);

        let if_expr = new_astref(ast::IfExpression {
            condition: condition.clone(),
            consequent: condition,
            alternate: if elements.is_empty() {
                new_astref(ast::Bool { value: false })
            } else {
                self.create_if_expr_from_disjunction(elements)
            },
        });
        if_expr
    }

    fn conjunction(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let mut children = Vec::new();

        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            children.push(self.expression(stream, false)?);
        }

        if children.is_empty() {
            return Err(ParseError::new("At least one expression required"));
        }

        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token

        Ok(self.create_if_expr_from_conjunction(&mut children))
    }

    fn create_if_expr_from_conjunction(&self, elements: &mut Vec<AstRef>) -> AstRef {
        if elements.is_empty() {
            return new_astref(ast::Bool { value: true });
        }

        let condition = elements.remove(0);

        let if_expr = new_astref(ast::IfExpression {
            condition: condition.clone(),
            consequent: if elements.is_empty() {
                condition
            } else {
                self.create_if_expr_from_conjunction(elements)
            },
            alternate: new_astref(ast::Bool { value: false }),
        });
        if_expr
    }

    fn cond_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let mut clauses = Vec::new();

        while let Some(_) = Self::peek_token(stream, &vec![&LeftParen, &LeftBrace, &LeftBracket]) {
            let (condition, consequent) = self.cond_clause(stream)?;
            clauses.push((condition, consequent));
        }

        if clauses.is_empty() {
            return Err(ParseError::new("At least one clause required"));
        }

        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token

        Ok(self.create_if_expr_from_cond_clauses(&mut clauses))
    }

    fn create_if_expr_from_cond_clauses(&self, clauses: &mut Vec<(AstRef, AstRef)>) -> AstRef {
        if clauses.is_empty() {
            return new_astref(ast::Nil {});
        }

        let (condition, consequent) = clauses.remove(0);

        let if_expr = new_astref(ast::IfExpression {
            condition,
            consequent,
            alternate: self.create_if_expr_from_cond_clauses(clauses),
        });
        if_expr
    }

    fn cond_clause(
        &self,
        stream: &mut BufferedStream<Token>,
    ) -> Result<(AstRef, AstRef), ParseError> {
        let opening_token = Self::next_token(stream, &vec![&LeftParen, &LeftBrace, &LeftBracket])?;

        let condition = self.expression(stream, false)?;
        let consequent = self.expression(stream, false)?;

        Self::next_token(
            stream,
            &vec![&Self::closing_token_type(&opening_token.token_type)],
        )?; // consume closing token

        Ok((condition, consequent))
    }

    fn if_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let condition = self.expression(stream, false)?;
        let consequent = self.expression(stream, false)?;
        let alternate = self.expression(stream, false)?;

        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token

        Ok(new_astref(ast::IfExpression {
            condition,
            consequent,
            alternate,
        }))
    }

    fn definition(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let token = Self::peek_token(
            stream,
            &vec![&Identifier, &LeftParen, &LeftBrace, &LeftBracket],
        )
        .ok_or(ParseError::new("Unexpected token in definition"))?;

        match token.token_type {
            Identifier => self.definition_with_name(stream, &token, &end_token_type),
            _ => self.definition_with_lambda(stream, &token, &end_token_type),
        }
    }

    fn definition_with_lambda(
        &self,
        stream: &mut BufferedStream<Token>,
        opening_token: &Token,
        def_end_token_type: &TokenType,
    ) -> Result<AstRef, ParseError> {
        let closing_token_type = Self::closing_token_type(&opening_token.token_type);
        Self::next_token(stream, &vec![])?;

        let name_token = Self::next_token(stream, &vec![&Identifier])?;
        let name = name_token.get_string_value().unwrap();
        let mut variadic: Option<String> = None;

        let mut parameters = Vec::new();
        while let Some(_) = Self::peek_token(stream, &vec![&Identifier]) {
            let token = Self::next_token(stream, &vec![&Identifier])?;
            if let Some(_) = Self::peek_token(stream, &vec![&Dot3]) {
                Self::next_token(stream, &vec![&Dot3])?; // consume variadic token
                variadic = Some(token.get_string_value().unwrap());
                break;
            }
            parameters.push(token.get_string_value().unwrap());
        }

        Self::next_token(stream, &vec![&closing_token_type])?; // consume closing token for parameters

        let body = self.block(stream, def_end_token_type)?;

        Ok(new_astref(ast::Definition {
            name: name.clone(),
            value: new_astref(ast::Lambda {
                name: Some(name.clone()),
                parameters,
                variadic,
                body,
            }),
        }))
    }

    fn definition_with_name(
        &self,
        stream: &mut BufferedStream<Token>,
        name_token: &Token,
        end_token_type: &TokenType,
    ) -> Result<AstRef, ParseError> {
        let name = name_token.get_string_value().unwrap();
        Self::next_token(stream, &vec![&Identifier])?; // consume opening token (name)

        let value = self.expression(stream, false)?;

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(new_astref(ast::Definition { name, value }))
    }

    fn struct_definition(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<AstRef, ParseError> {
        let token = Self::next_token(stream, &vec![&Identifier])?;
        let name = token.get_string_value().unwrap();

        let opening_token = Self::next_token(stream, &vec![&LeftParen, &LeftBrace, &LeftBracket])?;
        let closing_token_type = Self::closing_token_type(&opening_token.token_type);

        let mut fields = Vec::new();
        while Self::peek_token(stream, &vec![&closing_token_type]).is_none() {
            let token = Self::next_token(stream, &vec![&Identifier])?;
            fields.push(token.get_string_value().unwrap());
        }

        Self::next_token(stream, &vec![&closing_token_type])?;

        Self::next_token(stream, &vec![&end_token_type])?; // consume closing token

        Ok(new_astref(ast::StructDefinition { name, fields }))
    }
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub token: Option<Token>,
}

impl ParseError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
            token: None,
        }
    }

    pub fn with_token(message: &str, token: Token) -> Self {
        Self {
            message: message.to_string(),
            token: Some(token),
        }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.message
    }
}

#[cfg(test)]
mod tests {

    use crate::frontend::lexer::tokens::{Op, TokenType};

    use super::ast::*;

    #[test]
    fn test_parser_simple_types() {
        let parser = super::Parser::new();
        let code = r#"
            123 
            3,14 
            #true
            "Thomas"
            an-identifier
            absolute::name
            nil
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.children.len(), 7);

        let integer = &borrow_ast(&program.children[0]);
        let integer = downcast_ast::<Integer>(integer).unwrap();
        assert_eq!(integer.value, 123);

        let real = &borrow_ast(&program.children[1]);
        let real = downcast_ast::<Real>(real).unwrap();
        assert_eq!(real.value, 3.14);

        let boolean = &borrow_ast(&program.children[2]);
        let boolean = downcast_ast::<Bool>(boolean).unwrap();
        assert_eq!(boolean.value, true);

        let string = &borrow_ast(&program.children[3]);
        let string = downcast_ast::<Str>(string).unwrap();
        assert_eq!(string.value, "Thomas");

        let ident = &borrow_ast(&program.children[4]);
        let ident = downcast_ast::<Identifier>(ident).unwrap();
        assert_eq!(ident.value, "an-identifier");

        let absname = &borrow_ast(&program.children[5]);
        let absname = downcast_ast::<AbsoluteName>(absname).unwrap();
        assert_eq!(
            absname.segments,
            vec!["absolute".to_string(), "name".to_string()]
        );

        let nil = &borrow_ast(&program.children[6]);
        let nil = downcast_ast::<Nil>(nil);
        assert!(nil.is_some());
    }

    #[test]
    fn test_definition() {
        let parser = super::Parser::new();
        let code = r#"
            (def answer 42)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.children.len(), 1);

        let definition = &borrow_ast(&program.children[0]);
        let definition = downcast_ast::<Definition>(definition).unwrap();
        assert_eq!(definition.name, "answer");

        let integer = &borrow_ast(&definition.value);
        let integer = downcast_ast::<Integer>(integer).unwrap();
        assert_eq!(integer.value, 42);
    }

    #[test]
    fn test_struct_definition() {
        let parser = super::Parser::new();
        let code = r#"
            (def-struct person [name age])
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.children.len(), 1);

        let struct_def = &borrow_ast(&program.children[0]);
        let struct_def = downcast_ast::<StructDefinition>(struct_def).unwrap();
        assert_eq!(struct_def.name, "person");
        assert_eq!(
            struct_def.fields,
            vec!["name".to_string(), "age".to_string()]
        );
    }

    #[test]
    fn test_function_definition() {
        let parser = super::Parser::new();
        let code = r#"
            (def (do-something a b) (some-library-function a b))  
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();

        let definition = &borrow_ast(&program.children[0]);
        let definition = downcast_ast::<Definition>(definition).unwrap();
        assert_eq!(definition.name, "do-something");

        let lambda = &borrow_ast(&definition.value);
        let lambda = downcast_ast::<Lambda>(lambda).unwrap();
        assert_eq!(lambda.parameters, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_lambda() {
        let parser = super::Parser::new();
        let code = r#"
            (lambda (a b) (add a b))
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();

        let lambda = &borrow_ast(&program.children[0]);
        let lambda = downcast_ast::<Lambda>(lambda).unwrap();
        assert_eq!(*lambda.parameters, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn test_if_expression() {
        let parser = super::Parser::new();
        let code = r#"
            (if #t
                123
                456)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.children.len(), 1);

        let if_expr = &borrow_ast(&program.children[0]);
        let if_expr = downcast_ast::<IfExpression>(if_expr).unwrap();

        let condition = &borrow_ast(&if_expr.condition);
        let condition = downcast_ast::<Bool>(condition).unwrap();
        assert_eq!(condition.value, true);

        let consequent = &borrow_ast(&if_expr.consequent);
        let consequent = downcast_ast::<Integer>(consequent).unwrap();
        assert_eq!(consequent.value, 123);

        let alternate = &borrow_ast(&if_expr.alternate);
        let alternate = downcast_ast::<Integer>(alternate).unwrap();
        assert_eq!(alternate.value, 456);
    }

    #[test]
    fn test_cond_expression() {
        let parser = super::Parser::new();
        let code = r#"
            (cond
                [1 "Adult"]
                [2 "Teenager"]
                [#t "Child"])
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());
    }

    #[test]
    fn test_conjunction() {
        let parser = super::Parser::new();
        let code = r#"
            (and 1 2 3)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());
    }

    #[test]
    fn test_disjunction() {
        let parser = super::Parser::new();
        let code = r#"
            (or 1 2 3)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());
    }

    #[test]
    fn test_identifier() {
        let parser = super::Parser::new();
        let code = r#"
            (def answer 42)
            answer
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());

        let program = program.unwrap();

        let ident = &borrow_ast(&program.children[1]);
        let ident = downcast_ast::<Identifier>(ident).unwrap();
        assert_eq!(ident.value, "answer");
    }

    #[test]
    fn test_symbol() {
        let parser = super::Parser::new();
        let code = r#"
            'answer
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());

        let program = program.unwrap();

        let symbol = &borrow_ast(&program.children[0]);
        let symbol = downcast_ast::<Symbol>(symbol).unwrap();
        assert_eq!(symbol.value, "'answer");
    }

    #[test]
    fn test_call() {
        let parser = super::Parser::new();
        let code = r#"
            (+ 1 2)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());

        let program = program.unwrap();

        let call = &borrow_ast(&program.children[0]);
        let call = downcast_ast::<Call>(call).unwrap();

        let callee = &borrow_ast(&call.callee);
        let callee = downcast_ast::<Operator>(callee).unwrap();
        assert_eq!(callee.value, Op::Plus);
        assert_eq!(call.arguments.len(), 2);
    }

    #[test]
    fn test_quotation() {
        let parser = super::Parser::new();
        let code = r#"
            '(1 2 a)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());

        let program = program.unwrap();

        let list = &borrow_ast(&program.children[0]);
        let list = downcast_ast::<List>(list).unwrap();
        assert_eq!(list.elements.len(), 3);

        let integer = &borrow_ast(&list.elements[0]);
        let integer = downcast_ast::<Integer>(integer).unwrap();
        assert_eq!(integer.value, 1);

        let integer = &borrow_ast(&list.elements[1]);
        let integer = downcast_ast::<Integer>(integer).unwrap();
        assert_eq!(integer.value, 2);

        let ident = &borrow_ast(&list.elements[2]);
        let ident = downcast_ast::<Quote>(ident).unwrap();
        assert_eq!(ident.value.token_type, TokenType::Identifier);
        assert_eq!(ident.value.get_string_value().unwrap(), "a");
    }

    #[test]
    fn test_let_expression() {
        let parser = super::Parser::new();
        let code = r#"
            (let ((a 1) (b 2))
                (+ a b))
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());
        let program = program.unwrap();

        let block = &borrow_ast(&program.children[0]);
        let block = downcast_ast::<Block>(block).unwrap();
        assert_eq!(block.children.len(), 3);
    }

    #[test]
    fn test_var_param() {
        let parser = super::Parser::new();
        let code = r#"
            (def (add numbers...) 
                (+ ...numbers))
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());
        let program = program.unwrap();

        let def = &borrow_ast(&program.children[0]);
        let def = downcast_ast::<Definition>(def).unwrap();

        let lambda = &borrow_ast(&def.value);
        let lambda = downcast_ast::<Lambda>(lambda).unwrap();
        assert_eq!(lambda.variadic, Some("numbers".to_string()));
    }

    #[test]
    fn test_tail_call() {
        let parser = super::Parser::new();
        let code = r#"
            (def (factorial n) 
                (def (helper n acc) 
                    (if (= n 0)
                        acc
                        (helper (- n 1) (* acc n))))
                (helper n 1))
            (factorial 5)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());
        let program = program.unwrap();

        let def = &borrow_ast(&program.children[0]);
        let def = downcast_ast::<Definition>(def).unwrap();

        let factorial = &borrow_ast(&def.value);
        let factorial = downcast_ast::<Lambda>(factorial).unwrap();

        let factorial_body = &borrow_ast(&factorial.body);
        let factorial_body = downcast_ast::<Block>(factorial_body).unwrap();

        let helper_def = &borrow_ast(&factorial_body.children[0]);
        let helper_def = downcast_ast::<Definition>(helper_def).unwrap();

        let helper = &borrow_ast(&helper_def.value);
        let helper = downcast_ast::<Lambda>(helper).unwrap();

        let helper_body = &borrow_ast(&helper.body);
        let helper_body = downcast_ast::<Block>(helper_body).unwrap();

        let helper_if = &borrow_ast(&helper_body.children[0]);
        let helper_if = downcast_ast::<IfExpression>(helper_if).unwrap();

        let tail_call = &borrow_ast(&helper_if.alternate);
        let tail_call = downcast_ast::<Call>(tail_call).unwrap();

        assert!(tail_call.is_tail_call);
    }

    #[test]
    fn test_pair() {
        let parser = super::Parser::new();
        let code = r#"
        (1 . 2)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());
        let program = program.unwrap();

        let child0 = &borrow_ast(&program.children[0]);
        let pair = downcast_ast::<Pair>(child0).unwrap();

        let left = &borrow_ast(&pair.left);
        let left = downcast_ast::<Integer>(left).unwrap();
        assert_eq!(left.value, 1);

        let right = &borrow_ast(&pair.right);
        let right = downcast_ast::<Integer>(right).unwrap();
        assert_eq!(right.value, 2);
    }
}

use std::error::Error;

use super::lexer::stream::BufferedStream;
use super::lexer::tokens::{Token, TokenType, TokenType::*};
use super::lexer::Lexer;

pub mod ast;
pub mod visitor;

pub struct Parser {}

impl Parser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&self, code: &str) -> Result<ast::Program, ParseError> {
        let mut stream = BufferedStream::new(Box::new(Lexer::new(code)));
        self.program(&mut stream)
    }

    fn next_token(
        stream: &mut BufferedStream<Token>,
        expected_types: &Vec<TokenType>,
    ) -> Result<Token, ParseError> {
        let token = stream
            .next()
            .ok_or(ParseError::new("Unexpected end of input"))?;
        for expected_type in expected_types {
            if token.token_type == *expected_type {
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
        expected_types: &Vec<TokenType>,
    ) -> Option<Token> {
        let token = stream.peek()?;
        for expected_type in expected_types {
            if token.token_type == *expected_type {
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

        Ok(ast::Program { children })
    }

    fn expression(
        &self,
        stream: &mut BufferedStream<Token>,
        define_allowed: bool,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let token = Self::next_token(stream, &vec![])?;

        match token.token_type {
            Integer => Ok(Box::new(ast::Integer {
                value: token.get_int_value().unwrap(),
            })),
            Real => Ok(Box::new(ast::Real {
                value: token.get_real_value().unwrap(),
            })),
            Bool => Ok(Box::new(ast::Bool {
                value: token.get_bool_value().unwrap(),
            })),
            Str => Ok(Box::new(ast::Str {
                value: token.get_string_value().unwrap(),
            })),
            LeftParen | LeftBrace | LeftBracket => {
                self.symbolic_expression(&token, stream, define_allowed)
            }
            _ => Err(ParseError::with_token("Unexpected token", token)),
        }
    }

    fn closing_token_type(opening_token_type: &TokenType) -> TokenType {
        match opening_token_type {
            LeftParen => RightParen,
            LeftBrace => RightBrace,
            LeftBracket => RightBracket,
            _ => unreachable!(),
        }
    }

    fn symbolic_expression(
        &self,
        start_token: &Token,
        stream: &mut BufferedStream<Token>,
        define_allowed: bool,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let end_token_type = Self::closing_token_type(&start_token.token_type);

        let token = Self::next_token(stream, &vec![])?;

        match token.token_type {
            Def => {
                if define_allowed {
                    self.definition(stream, end_token_type)
                } else {
                    Err(ParseError::new("Definition not allowed here"))
                }
            }
            If => self.if_expression(stream, end_token_type),
            Cond => self.cond_expression(stream, end_token_type),
            _ => Err(ParseError::new("Unexpected token")),
        }
    }

    fn cond_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let mut clauses = Vec::new();

        while let Some(_) = Self::peek_token(stream, &vec![LeftParen, LeftBrace, LeftBracket]) {
            let (condition, consequent) = self.cond_clause(stream)?;
            clauses.push((condition, consequent));
        }

        if clauses.is_empty() {
            return Err(ParseError::new("At least one clause required"));
        }

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(self.create_if_expr_from_cond_clauses(&mut clauses))
    }

    fn create_if_expr_from_cond_clauses(
        &self,
        clauses: &mut Vec<(Box<dyn ast::Ast>, Box<dyn ast::Ast>)>,
    ) -> Box<dyn ast::Ast> {
        if clauses.is_empty() {
            return Box::new(ast::Nil {});
        }

        let (condition, consequent) = clauses.remove(0);

        let if_expr = Box::new(ast::IfExpression {
            condition,
            consequent,
            alternate: self.create_if_expr_from_cond_clauses(clauses),
        });
        if_expr
    }

    fn cond_clause(
        &self,
        stream: &mut BufferedStream<Token>,
    ) -> Result<(Box<dyn ast::Ast>, Box<dyn ast::Ast>), ParseError> {
        let opening_token = Self::next_token(stream, &vec![LeftParen, LeftBrace, LeftBracket])?;

        let condition = self.expression(stream, false)?;
        let consequent = self.expression(stream, false)?;

        Self::next_token(
            stream,
            &vec![Self::closing_token_type(&opening_token.token_type)],
        )?; // consume closing token

        Ok((condition, consequent))
    }

    fn if_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let condition = self.expression(stream, false)?;
        let consequent = self.expression(stream, false)?;
        let alternate = self.expression(stream, false)?;

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(Box::new(ast::IfExpression {
            condition,
            consequent,
            alternate,
        }))
    }

    fn definition(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let name_token = Self::next_token(stream, &vec![Identifier])?;
        let name = name_token.get_string_value().unwrap();

        let value = self.expression(stream, false)?;

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(Box::new(ast::Definition { name, value }))
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

    use super::ast::*;

    #[test]
    fn test_parser_simple_types() {
        let parser = super::Parser::new();
        let code = r#"
            123 
            3,14 
            #true
            "Thomas"
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.children.len(), 4);

        let integer = downcast_ast::<Integer>(&program.children[0]).unwrap();
        assert_eq!(integer.value, 123);

        let real = downcast_ast::<Real>(&program.children[1]).unwrap();
        assert_eq!(real.value, 3.14);

        let boolean = downcast_ast::<Bool>(&program.children[2]).unwrap();
        assert_eq!(boolean.value, true);

        let string = downcast_ast::<Str>(&program.children[3]).unwrap();
        assert_eq!(string.value, "Thomas");
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

        let definition = downcast_ast::<Definition>(&program.children[0]).unwrap();
        assert_eq!(definition.name, "answer");

        let integer = downcast_ast::<Integer>(&definition.value).unwrap();
        assert_eq!(integer.value, 42);
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

        let if_expr = downcast_ast::<IfExpression>(&program.children[0]).unwrap();
        let condition = downcast_ast::<Bool>(&if_expr.condition).unwrap();
        assert_eq!(condition.value, true);
        let consequent = downcast_ast::<Integer>(&if_expr.consequent).unwrap();
        assert_eq!(consequent.value, 123);
        let alternate = downcast_ast::<Integer>(&if_expr.alternate).unwrap();
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
}

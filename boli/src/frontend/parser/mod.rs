use std::error::Error;

use crate::frontend::parser::ast::downcast_ast;

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

    fn program(&self, stream: &mut BufferedStream<Token>) -> Result<ast::Program, ParseError> {
        let mut children = Vec::new();
        while let Ok(child) = self.expression(stream, true) {
            children.push(child);
        }

        Ok(ast::Program { children })
    }

    fn expression(
        &self,
        stream: &mut BufferedStream<Token>,
        define_allowed: bool,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let token = stream
            .next()
            .ok_or(ParseError::new("Unexpected end of input"))?;

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

    fn symbolic_expression(
        &self,
        start_token: &Token,
        stream: &mut BufferedStream<Token>,
        define_allowed: bool,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let end_token_type = match start_token.token_type {
            LeftParen => RightParen,
            LeftBrace => RightBrace,
            LeftBracket => RightBracket,
            _ => unreachable!(),
        };

        let token = stream
            .next()
            .ok_or(ParseError::new("Unexpected end of input"))?;

        match token.token_type {
            Def => {
                if define_allowed {
                    self.definition(stream, end_token_type)
                } else {
                    Err(ParseError::new("Definition not allowed here"))
                }
            }
            _ => Err(ParseError::new("Unexpected token")),
        }
    }

    fn definition(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Box<dyn ast::Ast>, ParseError> {
        let token = stream
            .next()
            .ok_or(ParseError::new("Unexpected end of input"))?;

        let name = match token.token_type {
            Identifier => token.get_string_value().unwrap(),
            _ => return Err(ParseError::new("Expected identifier")),
        };

        let value = self.expression(stream, false)?;
        if downcast_ast::<ast::Definition>(&value).is_some() {
            return Err(ParseError::new("Definition not allowed in definition"));
        }

        let end_token = stream
            .next()
            .ok_or(ParseError::new("Unexpected end of input"))?;
        if end_token.token_type != end_token_type {
            return Err(ParseError::new("Expected closing token"));
        }

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
            (def answer 42)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok());
        let program = program.unwrap();
        assert_eq!(program.children.len(), 5);

        let integer = downcast_ast::<Integer>(&program.children[0]).unwrap();
        assert_eq!(integer.value, 123);

        let real = downcast_ast::<Real>(&program.children[1]).unwrap();
        assert_eq!(real.value, 3.14);

        let boolean = downcast_ast::<Bool>(&program.children[2]).unwrap();
        assert_eq!(boolean.value, true);

        let string = downcast_ast::<Str>(&program.children[3]).unwrap();
        assert_eq!(string.value, "Thomas");

        let definition = downcast_ast::<Definition>(&program.children[4]).unwrap();
        assert_eq!(definition.name, "answer");

        let integer = downcast_ast::<Integer>(&definition.value).unwrap();
        assert_eq!(integer.value, 42);
    }
}

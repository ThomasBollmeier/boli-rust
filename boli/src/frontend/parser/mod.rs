use core::str;
use std::error::Error;
use std::rc::Rc;

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

        Ok(ast::Program { children })
    }

    fn expression(
        &self,
        stream: &mut BufferedStream<Token>,
        define_allowed: bool,
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
        let token = Self::next_token(stream, &vec![])?;

        match token.token_type {
            Integer => Ok(Rc::new(ast::Integer {
                value: token.get_int_value().unwrap(),
            })),
            Real => Ok(Rc::new(ast::Real {
                value: token.get_real_value().unwrap(),
            })),
            Bool => Ok(Rc::new(ast::Bool {
                value: token.get_bool_value().unwrap(),
            })),
            Str => Ok(Rc::new(ast::Str {
                value: token.get_string_value().unwrap(),
            })),
            Identifier => Ok(Rc::new(ast::Identifier {
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
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
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
            Conjunction => self.conjunction(stream, end_token_type),
            Disjunction => self.disjunction(stream, end_token_type),
            _ => {
                stream.push_back(token);
                self.call(stream, end_token_type)
            }
        }
    }

    fn call(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
        let callee = self.expression(stream, false)?;
        let mut arguments = Vec::new();

        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            arguments.push(self.expression(stream, false)?);
        }

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(Rc::new(ast::Call { callee, arguments }))
    }

    fn disjunction(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
        let mut children = Vec::new();

        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            children.push(self.expression(stream, false)?);
        }

        if children.is_empty() {
            return Err(ParseError::new("At least one expression required"));
        }

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(self.create_if_expr_from_disjunction(&mut children))
    }

    fn create_if_expr_from_disjunction(
        &self,
        elements: &mut Vec<Rc<dyn ast::Ast>>,
    ) -> Rc<dyn ast::Ast> {
        if elements.is_empty() {
            return Rc::new(ast::Bool { value: false });
        }

        let condition = elements.remove(0);

        let if_expr = Rc::new(ast::IfExpression {
            condition: condition.clone(),
            consequent: condition,
            alternate: if elements.is_empty() {
                Rc::new(ast::Bool { value: false })
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
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
        let mut children = Vec::new();

        while Self::peek_token(stream, &vec![&end_token_type]).is_none() {
            children.push(self.expression(stream, false)?);
        }

        if children.is_empty() {
            return Err(ParseError::new("At least one expression required"));
        }

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(self.create_if_expr_from_conjunction(&mut children))
    }

    fn create_if_expr_from_conjunction(
        &self,
        elements: &mut Vec<Rc<dyn ast::Ast>>,
    ) -> Rc<dyn ast::Ast> {
        if elements.is_empty() {
            return Rc::new(ast::Bool { value: true });
        }

        let condition = elements.remove(0);

        let if_expr = Rc::new(ast::IfExpression {
            condition: condition.clone(),
            consequent: if elements.is_empty() {
                condition
            } else {
                self.create_if_expr_from_conjunction(elements)
            },
            alternate: Rc::new(ast::Bool { value: false }),
        });
        if_expr
    }

    fn cond_expression(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
        let mut clauses = Vec::new();

        while let Some(_) = Self::peek_token(stream, &vec![&LeftParen, &LeftBrace, &LeftBracket]) {
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
        clauses: &mut Vec<(Rc<dyn ast::Ast>, Rc<dyn ast::Ast>)>,
    ) -> Rc<dyn ast::Ast> {
        if clauses.is_empty() {
            return Rc::new(ast::Nil {});
        }

        let (condition, consequent) = clauses.remove(0);

        let if_expr = Rc::new(ast::IfExpression {
            condition,
            consequent,
            alternate: self.create_if_expr_from_cond_clauses(clauses),
        });
        if_expr
    }

    fn cond_clause(
        &self,
        stream: &mut BufferedStream<Token>,
    ) -> Result<(Rc<dyn ast::Ast>, Rc<dyn ast::Ast>), ParseError> {
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
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
        let condition = self.expression(stream, false)?;
        let consequent = self.expression(stream, false)?;
        let alternate = self.expression(stream, false)?;

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(Rc::new(ast::IfExpression {
            condition,
            consequent,
            alternate,
        }))
    }

    fn definition(
        &self,
        stream: &mut BufferedStream<Token>,
        end_token_type: TokenType,
    ) -> Result<Rc<dyn ast::Ast>, ParseError> {
        let name_token = Self::next_token(stream, &vec![Identifier])?;
        let name = name_token.get_string_value().unwrap();

        let value = self.expression(stream, false)?;

        Self::next_token(stream, &vec![end_token_type])?; // consume closing token

        Ok(Rc::new(ast::Definition { name, value }))
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
        let ident = downcast_ast::<Identifier>(&program.children[1]).unwrap();
        assert_eq!(ident.value, "answer");
    }

    #[test]
    fn test_call() {
        let parser = super::Parser::new();
        let code = r#"
            (add 1 2)
        "#;
        let program = parser.parse(code);
        assert!(program.is_ok(), "{}", program.err().unwrap());

        let program = program.unwrap();
        let call = downcast_ast::<Call>(&program.children[0]).unwrap();
        let callee = downcast_ast::<Identifier>(&call.callee).unwrap();
        assert_eq!(callee.value, "add");
        assert_eq!(call.arguments.len(), 2);
    }
}

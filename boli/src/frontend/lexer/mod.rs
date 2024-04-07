pub mod stream;
pub mod tokens;

use std::collections::HashSet;
use stream::{BufferedStream, CharsStream, Stream};
use tokens::{
    LogicalOp, Token,
    TokenType::{self, *},
};

pub struct Lexer {
    stream: BufferedStream<char>,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(code: &str) -> Self {
        Self {
            stream: BufferedStream::new(Box::new(CharsStream::new(code))),
            line: 1,
            column: 1,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.stream.next();
        if c == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        c
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.stream.peek() {
            if c.is_whitespace() {
                self.next_char();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.next_char() {
            if c == '\n' {
                break;
            }
        }
    }

    fn scan_logical_operator(
        &mut self,
        first_char: char,
        line: usize,
        column: usize,
    ) -> Option<Token> {
        let second_char = self.stream.peek()?;
        let token_type = match (first_char, second_char) {
            ('>', '=') => {
                self.next_char();
                LogicalOperator(LogicalOp::Ge)
            }
            ('>', _) => LogicalOperator(LogicalOp::Gt),
            ('<', '=') => {
                self.next_char();
                LogicalOperator(LogicalOp::Le)
            }
            ('<', _) => LogicalOperator(LogicalOp::Lt),
            _ => unreachable!(),
        };

        Some(Token::new(token_type, line, column))
    }

    fn scan_number(&mut self, first_char: char, line: usize, column: usize) -> Option<Token> {
        let mut number = String::new();
        number.push(first_char);

        while let Some(c) = self.stream.peek() {
            if c.is_digit(10) {
                number.push(self.next_char()?);
            } else if c == '.' {
                self.next_char();
                continue; // . can be used for grouping digits
            } else {
                break;
            }
        }

        if self.stream.peek() == Some(',') {
            self.next_char();
            number.push('.');
            while let Some(c) = self.stream.peek() {
                if c.is_digit(10) {
                    number.push(self.next_char()?);
                } else if c == '.' {
                    self.next_char();
                    continue; // . can be used for grouping digits
                } else {
                    break;
                }
            }
            let number = number.parse::<f64>().unwrap();
            Some(Token::new_real(number, line, column))
        } else {
            let number = number.parse::<i64>().unwrap();
            Some(Token::new_int(number, line, column))
        }
    }

    fn scan_string(&mut self, line: usize, column: usize) -> Option<Token> {
        let mut string = String::new();
        let mut previous_char = None;
        let mut terminated = false;

        while let Some(c) = self.next_char() {
            if c == '"' {
                if previous_char == Some('\\') {
                    string.pop();
                } else {
                    terminated = true;
                    break;
                }
            }
            string.push(c);
            previous_char = Some(c);
        }

        if terminated {
            Some(Token::new_str(string, line, column))
        } else {
            Some(Token::new_error(string, line, column))
        }
    }

    fn scan_identifier(&mut self, first_char: char, line: usize, column: usize) -> Option<Token> {
        let invalid_start = HashSet::<char>::from_iter("!?.,*:".chars());

        if invalid_start.contains(&first_char) {
            return None;
        }

        let mut identifier = String::new();
        identifier.push(first_char);

        let invalid_chars = HashSet::<char>::from_iter("\"(){}[]/.:".chars());

        while let Some(c) = self.stream.peek() {
            if !c.is_whitespace() && !invalid_chars.contains(&c) {
                identifier.push(self.next_char()?);
            } else {
                break;
            }
        }

        let token = match identifier.as_str() {
            "def" => Token::new(Def, line, column),
            "def-struct" => Token::new(DefStruct, line, column),
            "if" => Token::new(If, line, column),
            "and" => Token::new(Conjunction, line, column),
            "or" => Token::new(Disjunction, line, column),
            "lambda" | "λ" => Token::new(Lambda, line, column),
            "nil" => Token::new(Nil, line, column),
            "block" => Token::new(Block, line, column),
            "cond" => Token::new(Cond, line, column),
            "let" => Token::new(Let, line, column),
            "#f" | "#false" => Token::new_bool(false, line, column),
            "#t" | "#true" => Token::new_bool(true, line, column),
            _ => Token::new_identifier(identifier, line, column),
        };

        Some(token)
    }
}

impl Stream<Token> for Lexer {
    fn next(&mut self) -> Option<Token> {
        loop {
            self.skip_whitespace();
            let ch = self.next_char()?;

            let line = self.line;
            let column = self.column;

            if ch == ';' {
                self.skip_line_comment();
                continue;
            }

            if let Some(token_type) = TokenType::from_char(ch) {
                return Some(Token::new(token_type, line, column));
            }

            if ch == '>' || ch == '<' {
                return self.scan_logical_operator(ch, line, column);
            }

            if ch.is_digit(10) {
                return self.scan_number(ch, line, column);
            }

            if ch == '"' {
                return self.scan_string(line, column);
            }

            if let Some(token) = self.scan_identifier(ch, line, column) {
                return Some(token);
            }

            return Some(Token::new_error(ch.to_string(), line, column));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::lexer::tokens::{LogicalOp, Op, TokenValue};

    use super::*;

    #[test]
    fn test_scan_single_char_tokens() {
        let code = "(){}[]+-*/^%=";
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(lexer.next().unwrap().token_type, RightParen);
        assert_eq!(lexer.next().unwrap().token_type, LeftBrace);
        assert_eq!(lexer.next().unwrap().token_type, RightBrace);
        assert_eq!(lexer.next().unwrap().token_type, LeftBracket);
        assert_eq!(lexer.next().unwrap().token_type, RightBracket);
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Plus));
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Minus));
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Asterisk));
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Slash));
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Caret));
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Percent));
        assert_eq!(
            lexer.next().unwrap().token_type,
            LogicalOperator(LogicalOp::Eq)
        );
        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_scan_logical_operators() {
        let code = ">>=<<=";
        let mut lexer = Lexer::new(code);
        assert_eq!(
            lexer.next().unwrap().token_type,
            LogicalOperator(LogicalOp::Gt)
        );
        assert_eq!(
            lexer.next().unwrap().token_type,
            LogicalOperator(LogicalOp::Ge)
        );
        assert_eq!(
            lexer.next().unwrap().token_type,
            LogicalOperator(LogicalOp::Lt)
        );
        assert_eq!(
            lexer.next().unwrap().token_type,
            LogicalOperator(LogicalOp::Le)
        );
        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_ignores_line_comments() {
        let code = "
        ;; this is a comment
        (+ ; another comment )
        ";
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Plus));
        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_scan_numbers() {
        let code = "(+ 41 1,0 1.000.000)";
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Plus));

        let int_token = lexer.next().unwrap();
        assert_eq!(int_token.token_type, Integer);
        assert_eq!(int_token.token_value, Some(TokenValue::Integer(41)));

        let real_token = lexer.next().unwrap();
        assert_eq!(real_token.token_type, Real);
        assert_eq!(real_token.token_value, Some(TokenValue::Real(1.0)));

        let int_token = lexer.next().unwrap();
        assert_eq!(int_token.token_type, Integer);
        assert_eq!(int_token.token_value, Some(TokenValue::Integer(1_000_000)));

        assert_eq!(lexer.next().unwrap().token_type, RightParen);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_scan_string() {
        let code = r#"(+ "hello, world" "hello \"world")"#;
        let mut lexer = Lexer::new(code);
        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(lexer.next().unwrap().token_type, Operator(Op::Plus));

        let str_token = lexer.next().unwrap();
        assert_eq!(str_token.token_type, Str);
        assert_eq!(
            str_token.token_value,
            Some(TokenValue::Str("hello, world".to_string()))
        );

        let str_token = lexer.next().unwrap();
        assert_eq!(str_token.token_type, Str);
        assert_eq!(
            str_token.token_value,
            Some(TokenValue::Str(r#"hello "world"#.to_string()))
        );

        assert_eq!(lexer.next().unwrap().token_type, RightParen);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_scan_identifiers() {
        let code = r#"an-identifier defined? :legal 
            def def-struct if and or lambda λ 
            nil block cond let let-alone #f #false #t #true"#;
        let mut lexer = Lexer::new(code);
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Identifier("an-identifier".to_string()))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Identifier("defined?".to_string()))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Error(":".to_string()))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Identifier("legal".to_string()))
        );
        assert_eq!(lexer.next().unwrap().token_type, Def);
        assert_eq!(lexer.next().unwrap().token_type, DefStruct);
        assert_eq!(lexer.next().unwrap().token_type, If);
        assert_eq!(lexer.next().unwrap().token_type, Conjunction);
        assert_eq!(lexer.next().unwrap().token_type, Disjunction);
        assert_eq!(lexer.next().unwrap().token_type, Lambda);
        assert_eq!(lexer.next().unwrap().token_type, Lambda);
        assert_eq!(lexer.next().unwrap().token_type, Nil);
        assert_eq!(lexer.next().unwrap().token_type, Block);
        assert_eq!(lexer.next().unwrap().token_type, Cond);
        assert_eq!(lexer.next().unwrap().token_type, Let);
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Identifier("let-alone".to_string()))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Bool(false))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Bool(false))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Bool(true))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Bool(true))
        );
        assert!(lexer.next().is_none());
    }
}

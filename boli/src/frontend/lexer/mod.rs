pub mod stream;
pub mod tokens;

use std::collections::HashSet;
use stream::{BufferedStream, CharsStream, Stream};
use tokens::{
    LogicalOp, Token,
    TokenType::{self, *},
};

use self::tokens::Op;

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
            column: 0,
        }
    }

    fn next_char(&mut self) -> Option<char> {
        let c = self.stream.next();
        if c == Some('\n') {
            self.line += 1;
            self.column = 0;
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
        let mut is_absolute_name = false;

        let invalid_chars = HashSet::<char>::from_iter("\"(){}[]/.:".chars());

        loop {
            while let Some(c) = self.stream.peek() {
                if !c.is_whitespace() && !invalid_chars.contains(&c) {
                    identifier.push(self.next_char()?);
                } else {
                    break;
                }
            }

            let next_chars = self.stream.peek_many(3).iter().collect::<String>();
            if next_chars.len() < 3 {
                break;
            }

            if &next_chars.chars().take(2).collect::<String>() == "::"
                && !invalid_start.contains(&next_chars.chars().nth(2).unwrap())
            {
                is_absolute_name = true;
                self.next_char();
                self.next_char();
                self.next_char();
                identifier.push_str(&next_chars);
            } else {
                break;
            }
        }

        let token = match identifier.as_str() {
            "def" => Token::new(Def, line, column),
            "def-struct" => Token::new(DefStruct, line, column),
            "set!" => Token::new(SetBang, line, column),
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
            _ => {
                if !is_absolute_name {
                    Token::new_identifier(identifier, line, column)
                } else {
                    Token::new_absolute_name(identifier, line, column)
                }
            }
        };

        Some(token)
    }

    fn scan_quote(&mut self, ch: char, line: usize, column: usize) -> Option<Token> {
        let next_char = match self.next_char() {
            Some(c) => c,
            None => return Some(Token::new_error(ch.to_string(), line, column)),
        };

        match next_char {
            '(' => Some(Token::new(QuoteParen, line, column)),
            '{' => Some(Token::new(QuoteBrace, line, column)),
            '[' => Some(Token::new(QuoteBracket, line, column)),
            _ => {
                if let Some(ident) = self.scan_identifier(next_char, line, column) {
                    let symbol = format!("{}{}", ch, ident.get_string_value().unwrap());
                    Some(Token::new_symbol(symbol, line, column))
                } else {
                    Some(Token::new_error(ch.to_string(), line, column))
                }
            }
        }
    }

    fn scan_dot3(&mut self, line: usize, column: usize) -> Option<Token> {
        let next_chars = self.stream.peek_many(2).iter().collect::<String>();
        if next_chars == ".." {
            self.next_char();
            self.next_char();
            Some(Token::new(Dot3, line, column))
        } else if next_chars.len() > 0 && next_chars.chars().next().unwrap().is_whitespace() {
            Some(Token::new(Dot, line, column))
        } else {
            None
        }
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

            if ch == '+' || ch == '-' {
                let next_char = self.stream.peek().unwrap_or(' ');
                if next_char.is_digit(10) {
                    return self.scan_number(ch, line, column);
                } else if next_char.is_whitespace() {
                    match ch {
                        '+' => return Some(Token::new(Operator(Op::Plus), line, column)),
                        '-' => return Some(Token::new(Operator(Op::Minus), line, column)),
                        _ => unreachable!(),
                    }
                }
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

            if ch == '\'' {
                return self.scan_quote(ch, line, column);
            }

            if ch == '.' {
                if let Some(token) = self.scan_dot3(line, column) {
                    return Some(token);
                }
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
        let code = "(){}[]+ - */^%=";
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

    #[test]
    fn test_scan_absolute_name() {
        let code = r#"my-module::a-function"#;
        let mut lexer = Lexer::new(code);

        let abs_name_token = lexer.next().unwrap();
        assert_eq!(abs_name_token.token_type, AbsoluteName);

        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_scan_invalid_absolute_name() {
        let code = r#"my-module::"#;
        let mut lexer = Lexer::new(code);

        let ident_token = lexer.next().unwrap();
        assert_eq!(ident_token.token_type, Identifier);

        let error_token = lexer.next().unwrap();
        assert_eq!(error_token.token_type, Error);
        assert_eq!(
            error_token.token_value,
            Some(TokenValue::Error(":".to_string()))
        );
    }

    #[test]
    fn test_quotation() {
        let code = r#"'a '{1 2 3}"#;
        let mut lexer = Lexer::new(code);

        let symbol_token = lexer.next().unwrap();
        assert_eq!(symbol_token.token_type, Symbol);
        assert_eq!(
            symbol_token.token_value,
            Some(TokenValue::Symbol("'a".to_string()))
        );

        let quote_token = lexer.next().unwrap();
        assert_eq!(quote_token.token_type, QuoteBrace);

        let int_token = lexer.next().unwrap();
        assert_eq!(int_token.token_type, Integer);
        assert_eq!(int_token.token_value, Some(TokenValue::Integer(1)));

        let int_token = lexer.next().unwrap();
        assert_eq!(int_token.token_type, Integer);
        assert_eq!(int_token.token_value, Some(TokenValue::Integer(2)));

        let int_token = lexer.next().unwrap();
        assert_eq!(int_token.token_type, Integer);
        assert_eq!(int_token.token_value, Some(TokenValue::Integer(3)));

        let paren_token = lexer.next().unwrap();
        assert_eq!(paren_token.token_type, RightBrace);

        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_var_param() {
        let code = r#"(calc-sum numbers...)"#;
        let mut lexer = Lexer::new(code);

        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Identifier("calc-sum".to_string()))
        );
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Identifier("numbers".to_string()))
        );
        assert_eq!(lexer.next().unwrap().token_type, Dot3);
        assert_eq!(lexer.next().unwrap().token_type, RightParen);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_scan_pair() {
        let code = r#"(1 . 2)"#;
        let mut lexer = Lexer::new(code);

        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(lexer.next().unwrap().token_type, Integer);
        assert_eq!(lexer.next().unwrap().token_type, Dot);
        assert_eq!(lexer.next().unwrap().token_type, Integer);
        assert_eq!(lexer.next().unwrap().token_type, RightParen);
        assert!(lexer.next().is_none());
    }

    #[test]
    fn test_scan_set_bang() {
        let code = r#"(set! x 42)"#;
        let mut lexer = Lexer::new(code);

        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(lexer.next().unwrap().token_type, SetBang);
        assert_eq!(
            lexer.next().unwrap().token_value,
            Some(TokenValue::Identifier("x".to_string()))
        );
        assert_eq!(lexer.next().unwrap().token_type, Integer);
        assert_eq!(lexer.next().unwrap().token_type, RightParen);
        assert!(lexer.next().is_none());
    }
}

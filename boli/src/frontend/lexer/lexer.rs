use super::{
    stream::{BufferedStream, CharsStream, Stream},
    tokens::{
        LogicalOp, Token,
        TokenType::{self, *},
    },
};

struct Lexer {
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
            Some(Token {
                token_type: Real(number),
                line,
                column,
            })
        } else {
            let number = number.parse::<i64>().unwrap();
            Some(Token {
                token_type: Integer(number),
                line,
                column,
            })
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
                return Some(Token {
                    token_type,
                    line,
                    column,
                });
            }

            if ch == '>' {
                let token_type = if let Some('=') = self.stream.peek() {
                    self.next_char();
                    LogicalOperator(LogicalOp::Ge)
                } else {
                    LogicalOperator(LogicalOp::Gt)
                };

                return Some(Token {
                    token_type,
                    line,
                    column,
                });
            }

            if ch == '<' {
                let token_type = if let Some('=') = self.stream.peek() {
                    self.next_char();
                    LogicalOperator(LogicalOp::Le)
                } else {
                    LogicalOperator(LogicalOp::Lt)
                };

                return Some(Token {
                    token_type,
                    line,
                    column,
                });
            }

            if ch.is_digit(10) {
                return self.scan_number(ch, line, column);
            }

            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::frontend::lexer::tokens::{LogicalOp, Op};

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
        assert_eq!(lexer.next().unwrap().token_type, Integer(41));
        assert_eq!(lexer.next().unwrap().token_type, Real(1.0));
        assert_eq!(lexer.next().unwrap().token_type, Integer(1_000_000));
        assert_eq!(lexer.next().unwrap().token_type, RightParen);
        assert!(lexer.next().is_none());
    }
}

use super::{
    stream::{BufferedStream, CharsStream, Stream},
    tokens::{
        Token,
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
}

impl Stream<Token> for Lexer {
    fn next(&mut self) -> Option<Token> {
        loop {
            self.skip_whitespace();
            let ch = self.next_char()?;

            if let Some(token_type) = TokenType::from_char(ch) {
                return Some(Token {
                    token_type,
                    line: self.line,
                    column: self.column,
                });
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
    fn test_scan_parentheses() {
        let mut lexer = Lexer::new("()");
        assert_eq!(lexer.next().unwrap().token_type, LeftParen);
        assert_eq!(lexer.next().unwrap().token_type, RightParen);
    }

    #[test]
    fn test_single_char_tokens() {
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
}

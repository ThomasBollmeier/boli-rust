use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Op {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Caret,
    Percent,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalOp {
    Eq,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Identifier,
    Symbol,
    QuoteParen,
    QuoteBrace,
    QuoteBracket,
    Integer,
    Real,
    Bool,
    Str,
    Def,
    DefStruct,
    If,
    Conjunction,
    Disjunction,
    Lambda,
    Operator(Op),
    LogicalOperator(LogicalOp),
    Nil,
    Dot3,
    Block,
    Cond,
    Let,
    ModuleSep,
    Error,
}

impl TokenType {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '(' => Some(Self::LeftParen),
            ')' => Some(Self::RightParen),
            '{' => Some(Self::LeftBrace),
            '}' => Some(Self::RightBrace),
            '[' => Some(Self::LeftBracket),
            ']' => Some(Self::RightBracket),
            '+' => Some(Self::Operator(Op::Plus)),
            '-' => Some(Self::Operator(Op::Minus)),
            '*' => Some(Self::Operator(Op::Asterisk)),
            '/' => Some(Self::Operator(Op::Slash)),
            '^' => Some(Self::Operator(Op::Caret)),
            '%' => Some(Self::Operator(Op::Percent)),
            '=' => Some(Self::LogicalOperator(LogicalOp::Eq)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    Integer(i64),
    Real(f64),
    Bool(bool),
    Str(String),
    Symbol(String),
    Identifier(String),
    Error(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub token_value: Option<TokenValue>,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            token_value: None,
            line,
            column,
        }
    }

    pub fn new_int(value: i64, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Integer,
            token_value: Some(TokenValue::Integer(value)),
            line,
            column,
        }
    }

    pub fn new_real(value: f64, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Real,
            token_value: Some(TokenValue::Real(value)),
            line,
            column,
        }
    }

    pub fn new_bool(value: bool, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Bool,
            token_value: Some(TokenValue::Bool(value)),
            line,
            column,
        }
    }

    pub fn new_str(value: String, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Str,
            token_value: Some(TokenValue::Str(value)),
            line,
            column,
        }
    }

    pub fn new_symbol(value: String, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Symbol,
            token_value: Some(TokenValue::Symbol(value)),
            line,
            column,
        }
    }

    pub fn new_identifier(value: String, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Identifier,
            token_value: Some(TokenValue::Identifier(value)),
            line,
            column,
        }
    }

    pub fn new_error(value: String, line: usize, column: usize) -> Self {
        Self {
            token_type: TokenType::Error,
            token_value: Some(TokenValue::Error(value)),
            line,
            column,
        }
    }

    pub fn get_int_value(&self) -> Option<i64> {
        match self.token_value {
            Some(TokenValue::Integer(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_real_value(&self) -> Option<f64> {
        match self.token_value {
            Some(TokenValue::Real(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_bool_value(&self) -> Option<bool> {
        match self.token_value {
            Some(TokenValue::Bool(value)) => Some(value),
            _ => None,
        }
    }

    pub fn get_string_value(&self) -> Option<String> {
        match self.token_value {
            Some(TokenValue::Str(ref value)) => Some(value.to_string()),
            Some(TokenValue::Identifier(ref value)) => Some(value.to_string()),
            Some(TokenValue::Symbol(ref value)) => Some(value.to_string()),
            Some(TokenValue::Error(ref value)) => Some(value.to_string()),
            _ => None,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.token_value {
            Some(TokenValue::Integer(value)) => {
                write!(f, "Token: {:?}({})", self.token_type, value)
            }
            Some(TokenValue::Real(value)) => write!(f, "Token: {:?}({})", self.token_type, value),
            Some(TokenValue::Bool(value)) => write!(f, "Token: {:?}({})", self.token_type, value),
            Some(TokenValue::Str(value)) => write!(f, "Token: {:?}({})", self.token_type, value),
            Some(TokenValue::Symbol(value)) => write!(f, "Token: {:?}({})", self.token_type, value),
            Some(TokenValue::Identifier(value)) => {
                write!(f, "Token: {:?}({})", self.token_type, value)
            }
            Some(TokenValue::Error(value)) => write!(f, "Token, {:?}({})", self.token_type, value),
            None => write!(f, "Token: {:?}", self.token_type),
        }
    }
}

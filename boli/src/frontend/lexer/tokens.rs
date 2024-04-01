#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Caret,
    Percent,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogicalOp {
    Eq,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Identifier(String),
    Symbol(String),
    Quote,
    Integer(i64),
    Real(f64),
    Bool(bool),
    String(String),
    Def,
    DefStruct,
    If,
    Lambda,
    Operator(Op),
    LogicalOperator(LogicalOp),
    Nil,
    Dot3,
    Block,
    Cond,
    Let,
    ModuleSep,
    Unknown,
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

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self {
            token_type,
            line,
            column,
        }
    }
}

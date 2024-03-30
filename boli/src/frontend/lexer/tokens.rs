#[derive(Debug)]
pub enum Op {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Caret,
    Percent,
}

#[derive(Debug)]
pub enum LogicalOp {
    Eq,
    Gt,
    Ge,
    Lt,
    Le,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

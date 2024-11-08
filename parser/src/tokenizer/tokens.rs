use super::Cursor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Eq,
    Plus,
    Minus,
    Star,
    Bar,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    Let,
    Func,
    Identifier(String),
    IntLit(String),
    FloatLit(String),
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    SemiColon,
    Operator(Operator),
    Eof,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    line: usize,
    column: usize,
    pub kind: TokenKind,
}
impl Token {
    pub fn func(cursor: &Cursor) -> Self {
        Self::new(TokenKind::Func, cursor)
    }
    pub fn identifier(buf: String, cursor: &Cursor) -> Self {
        Self::new(TokenKind::Identifier(buf), cursor)
    }
    pub fn let_token(cursor: &Cursor) -> Self {
        Self::new(TokenKind::Let, cursor)
    }
    pub fn float_lit(buf: String, cursor: &Cursor) -> Self {
        Self::new(TokenKind::FloatLit(buf), cursor)
    }
    pub fn int_lit(buf: String, cursor: &Cursor) -> Self {
        Self::new(TokenKind::IntLit(buf), cursor)
    }
    pub fn new(kind: TokenKind, cursor: &Cursor) -> Self {
        Self {
            kind,
            line: cursor.line(),
            column: cursor.column(),
        }
    }
    pub fn refkind(&self) -> &TokenKind {
        &self.kind
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn column(&self) -> usize {
        self.column
    }
}

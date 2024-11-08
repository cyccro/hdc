use crate::tokenizer::Operator;

#[derive(Debug, Clone)]
pub enum LetDeclKind {
    Normal,
}
#[derive(Debug, Clone)]
pub enum Expression {
    Program(Vec<Expression>),
    LetDecl {
        kind: LetDeclKind,
        varname: String,
        expr: Box<Expression>,
    },
    BinExpr {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        op: Operator,
    },
    Negative(Box<Expression>),
    Identifier(String),
    IntLit(String),
    FloatLit(String),
}

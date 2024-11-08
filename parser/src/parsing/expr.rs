use crate::tokenizer::Operator;

#[derive(Debug)]
pub enum LetDeclKind {
    Normal,
}
#[derive(Debug)]
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
    Identifier(String),
    IntLit(String),
    FloatLit(String),
}

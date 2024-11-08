use crate::tokenizer::Operator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Clone)]
pub enum LetDeclKind {
    Normal,
}
#[derive(Debug, Clone)]
pub enum Expression {
    Program(Vec<Expression>),
    Block(Vec<Expression>),
    FuncDecl {
        identifier: String,
        params: Vec<Param>,
        rtype: Option<String>,
        block: Box<Expression>,
    },
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

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
    Identifier(String),
    IntLit(String),
    FloatLit(String),
}

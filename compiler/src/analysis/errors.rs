use super::SemanticType;

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndeclaredVariable(String),
    UnrecognizedType(String),
    FunctionRedeclare(String),
    ProgramAnalysis,
    InvalidBinExpr {
        lhs_type: SemanticType,
        rhs_type: SemanticType,
    },
    InvalidFnType {
        return_type: SemanticType,
        block_type: SemanticType,
    },
}

use super::SemanticType;

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndeclaredVariable,
    ProgramAnalysis,
    InvalidBinExpr {
        lhs_type: SemanticType,
        rhs_type: SemanticType,
    },
}


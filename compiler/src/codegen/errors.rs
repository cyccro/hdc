use std::num::{ParseFloatError, ParseIntError};

use inkwell::values::BasicValueEnum;
use parser::{
    parsing::{Expression, ParsingError},
    tokenizer::TokenizationError,
};

use crate::analysis::errors::SemanticError;

#[derive(Debug, Clone)]
pub enum LitParseError {
    Int(ParseIntError),
    Float(ParseFloatError),
}

#[derive(Debug, Clone)]
pub enum CompilationError {
    Tokenization(TokenizationError),
    Parsing(ParsingError),
    TypeError(SemanticError),
    LitParseError(LitParseError),
    UndeclaredVariable(String),
    InvalidNegation(Expression),
    TryingAssignVoid,
}

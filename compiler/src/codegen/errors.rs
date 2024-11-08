use std::num::{ParseFloatError, ParseIntError};

use parser::{parsing::ParsingError, tokenizer::TokenizationError};

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
}

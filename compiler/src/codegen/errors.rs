use std::{
    collections::VecDeque,
    num::{ParseFloatError, ParseIntError},
};

use parser::{
    parsing::{Expression, ParseStep, ParsingError},
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
    Parsing(ParsingError, VecDeque<ParseStep>),
    TypeError(SemanticError),
    LitParseError(LitParseError),
    UndeclaredVariable(String),
    InvalidNegation(Expression),
    InvalidRedeclare(String),
    TryingAssignVoid,
}
impl std::fmt::Display for CompilationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::InvalidRedeclare(e) => {
                write!(f, "Cannot redeclare variable called {e}")
            }
            CompilationError::Tokenization(e) => write!(f, "Tokenization Error: {e:?}"),
            CompilationError::Parsing(e, backtrace) => {
                write!(f, "Parsing error: {e:?}\n Parsing Backtrace: [\n{}", {
                    let mut buffer = String::new();
                    for step in backtrace {
                        buffer.push_str(&format!("  {step},\n"));
                    }
                    buffer.push(']');
                    buffer
                })
            }
            CompilationError::TryingAssignVoid => write!(
                f,
                "DIdnt implement yet, but somewhere in the code is trying to assign to void"
            ),
            CompilationError::LitParseError(e) => write!(f, "Invalid Literal: {e:?}"),
            CompilationError::TypeError(e) => write!(f, "TypeError: {e:?}"),
            CompilationError::UndeclaredVariable(v) => write!(f, "Undeclared variable named: {v}"),
            CompilationError::InvalidNegation(e) => write!(f, "Invalid use of unary operator"),
        }
    }
}

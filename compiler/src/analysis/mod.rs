pub mod errors;
use parser::parsing::Expression;
use std::collections::HashMap;

use self::errors::SemanticError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Int32,
    Float32,
}
#[derive(Debug)]
pub struct SemanticAnalayzer {
    variables: HashMap<String, SemanticType>,
}
impl SemanticAnalayzer {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }
    pub fn create_var(
        &mut self,
        varname: &String,
        expr: &Expression,
    ) -> Result<SemanticType, SemanticError> {
        let stype = self.analyze_expr(expr)?;
        self.variables
            .insert(varname.clone(), stype.clone())
            .unwrap();
        Ok(stype)
    }
    pub fn analyze_var(&self, varname: &String) -> Result<&SemanticType, SemanticError> {
        self.variables
            .get(varname)
            .ok_or(SemanticError::UndeclaredVariable)
    }
    pub fn analyze_binexpr(
        &mut self,
        lhs: &Expression,
        rhs: &Expression,
    ) -> Result<SemanticType, SemanticError> {
        let lhs = self.analyze_expr(lhs)?;
        let rhs = self.analyze_expr(rhs)?;
        if lhs == rhs {
            Ok(lhs)
        } else {
            Err(SemanticError::InvalidBinExpr {
                lhs_type: lhs,
                rhs_type: rhs,
            })
        }
    }
    pub fn analyze_expr(&mut self, expr: &Expression) -> Result<SemanticType, SemanticError> {
        Ok(match expr {
            Expression::IntLit(_) => SemanticType::Int32,
            Expression::FloatLit(_) => SemanticType::Float32,
            Expression::LetDecl { varname, expr, .. } => self.create_var(varname, &**expr)?,
            Expression::Identifier(s) => self.analyze_var(s)?.clone(),
            Expression::Program(_) => return Err(SemanticError::ProgramAnalysis),
            Expression::BinExpr { lhs, rhs, .. } => self.analyze_binexpr(&**lhs, &**rhs)?,
            Expression::Negative(expr) => self.analyze_expr(&**expr)?,
        })
    }
}

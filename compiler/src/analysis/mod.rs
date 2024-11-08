pub mod errors;
use parser::parsing::{Expression, Param};
use std::collections::HashMap;

use self::errors::SemanticError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticType {
    Int32,
    Float32,
    Void,
    FnType {
        params: Vec<SemanticType>,
        rtype: Box<SemanticType>,
    },
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
    pub fn get_type(s: Option<&str>) -> Result<SemanticType, SemanticError> {
        let Some(s) = s else {
            return Ok(SemanticType::Void);
        };
        Ok(match s {
            "int32" => SemanticType::Int32,
            "f32" => SemanticType::Float32,
            "void" => SemanticType::Void,
            _ => return Err(SemanticError::UnrecognizedType(s.to_string())),
        })
    }
    pub fn delete_var(&mut self, varname: &String) -> Option<SemanticType> {
        self.variables.remove(varname)
    }
    pub fn create_var(
        &mut self,
        varname: &String,
        expr: &Expression,
    ) -> Result<(SemanticType, Option<SemanticType>), SemanticError> {
        let stype = self.analyze_expr(expr)?;
        let old_type = self.variables.insert(varname.clone(), stype.clone());
        Ok((stype, old_type))
    }
    pub fn analyze_var(&self, varname: &String) -> Result<&SemanticType, SemanticError> {
        self.variables
            .get(varname)
            .ok_or(SemanticError::UndeclaredVariable(varname.clone()))
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
            Expression::LetDecl { varname, expr, .. } => self.create_var(varname, &**expr)?.0,
            Expression::Identifier(s) => self.analyze_var(s)?.clone(),
            Expression::Program(_) => return Err(SemanticError::ProgramAnalysis),
            Expression::BinExpr { lhs, rhs, .. } => self.analyze_binexpr(&**lhs, &**rhs)?,
            Expression::Negative(expr) => self.analyze_expr(&**expr)?,
            Expression::Block(exprs) => {
                if let Some((last, rest)) = exprs.split_last() {
                    for expr in rest {
                        self.analyze_expr(expr)?;
                    }
                    self.analyze_expr(last)?
                } else {
                    SemanticType::Int32
                }
            }
            Expression::FuncDecl {
                identifier,
                params,
                rtype,
                block,
            } => {
                let rtype = Self::get_type(rtype.as_deref())?;
                let block_type = self.analyze_expr(&**block)?;
                if block_type == rtype {
                    let params = {
                        let mut parameters = Vec::with_capacity(params.len());
                        for param in params {
                            parameters.push(Self::get_type(Some(&param.kind))?);
                        }
                        parameters
                    };
                    let ftype = SemanticType::FnType {
                        params,
                        rtype: Box::new(rtype),
                    };
                    if let Some(_) = self.variables.insert(identifier.to_string(), ftype.clone()) {
                        return Err(SemanticError::FunctionRedeclare(identifier.clone()));
                    };
                    ftype
                } else {
                    return Err(SemanticError::InvalidFnType {
                        return_type: rtype,
                        block_type,
                    });
                }
            }
        })
    }
}

use std::{collections::HashMap, path::Path};

use super::errors::CompilationError;
use crate::analysis::{errors::SemanticError, SemanticAnalayzer, SemanticType};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{
        AnyType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FloatType, IntType, VoidType,
    },
    values::{BasicValue, BasicValueEnum, PointerValue},
    AddressSpace,
};
use parser::{
    parsing::{Expression, LetDeclKind},
    tokenizer::Operator,
};

#[derive(Debug)]
pub struct CodeGenerator<'a> {
    builder: Builder<'a>,
    module: Module<'a>,
    context: &'a Context,
    analyzer: SemanticAnalayzer,
    variables: HashMap<String, PointerValue<'a>>,
}

impl<'a> CodeGenerator<'a> {
    pub fn create_ctx() -> Context {
        Context::create()
    }
    pub fn new(context: &'a Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("hdc");
        let f = module.add_function("main", context.void_type().fn_type(&[], false), None);
        let entry = context.append_basic_block(f, "entry");
        builder.position_at_end(entry);

        Self {
            context,
            builder,
            module,
            analyzer: SemanticAnalayzer::new(),
            variables: HashMap::new(),
        }
    }
    pub fn compile_source(
        &mut self,
        source: String,
        output: Option<&Path>,
    ) -> Result<Vec<u8>, CompilationError> {
        let mut tokens = parser::tokenizer::Tokenizer::new(source)
            .gen()
            .map_err(|e| CompilationError::Tokenization(e))?;
        let ast = parser::parsing::Parser::new()
            .parse_tokens(&mut tokens)
            .map_err(|e| CompilationError::Parsing(e))?;
        self.compile_ast(ast)?;
        if let Some(path) = output {
            self.module.print_to_file(path).unwrap();
        }
        Ok(self.module.print_to_string().to_bytes().to_vec())
    }
    pub fn i32(&self) -> IntType<'a> {
        self.context.i32_type()
    }
    pub fn f32(&self) -> FloatType<'a> {
        self.context.f32_type()
    }
    pub fn void(&self) -> VoidType<'a> {
        self.context.void_type()
    }
    fn load(&self, vname: &String) -> Result<BasicValueEnum<'a>, CompilationError> {
        let varptr = self
            .variables
            .get(vname)
            .ok_or(CompilationError::UndeclaredVariable(vname.clone()))?;
        Ok(self
            .builder
            .build_load(*varptr, &format!("load-{vname}"))
            .unwrap())
    }
    fn type_from_stype(&self, stype: &SemanticType) -> Option<BasicTypeEnum<'a>> {
        Some(match stype {
            SemanticType::Int32 => self.i32().as_basic_type_enum(),
            SemanticType::Float32 => self.f32().as_basic_type_enum(),
            SemanticType::Void => return None,
            SemanticType::FnType { params, rtype } => {
                let params = {
                    let mut param_types = Vec::with_capacity(params.len());
                    for param in params {
                        if let Some(ptype) = self.type_from_stype(param) {
                            param_types.push(ptype.into())
                        } else {
                            continue;
                        }
                    }
                    param_types
                };
                if let Some(rtype) = self.type_from_stype(&**rtype) {
                    rtype.fn_type(&params, false)
                } else {
                    self.void().fn_type(&params, false)
                }
                .ptr_type(AddressSpace::default())
                .as_basic_type_enum()
            }
        })
    }
    fn type_based_on_semantics(
        &mut self,
        expr: &Expression,
    ) -> Result<Option<BasicTypeEnum<'a>>, CompilationError> {
        let stype = self
            .analyzer
            .analyze_expr(expr)
            .map_err(|e| CompilationError::TypeError(e))?;
        Ok(self.type_from_stype(&stype))
    }
    fn compile_ast(
        &mut self,
        expr: Expression,
    ) -> Result<Option<BasicValueEnum<'a>>, CompilationError> {
        Ok(match expr {
            Expression::IntLit(s) => Some(
                self.i32()
                    .const_int(s.parse::<u64>().unwrap(), false)
                    .as_basic_value_enum(),
            ),
            Expression::FloatLit(s) => Some(
                self.f32()
                    .const_float(s.parse::<f64>().unwrap())
                    .as_basic_value_enum(),
            ),
            Expression::LetDecl {
                kind,
                varname,
                expr,
            } => Some(
                self.compile_vardecl(kind, &varname, *expr)?
                    .as_basic_value_enum(),
            ),
            Expression::Program(mut exprs) => {
                let last_expr = exprs.pop();
                for expr in exprs {
                    self.compile_ast(expr)?;
                }
                return if let Some(last) = last_expr {
                    self.compile_ast(last)
                } else {
                    Ok(None)
                };
            }
            Expression::Identifier(s) => Some(self.load(&s)?.as_basic_value_enum()),
            Expression::BinExpr { lhs, rhs, op } => {
                let stype = self
                    .analyzer
                    .analyze_binexpr(&lhs, &rhs)
                    .map_err(|e| CompilationError::TypeError(e))?;
                Some(self.compile_binexpr(lhs, rhs, op, stype)?)
            }
            Expression::Negative(expr) => self.compile_negative(*expr)?,
            Expression::Block(exprs) => self.compile_block(exprs)?,
            Expression::FuncDecl {
                identifier,
                params,
                rtype,
                block,
            } => todo!(),
        })
    }
    fn compile_block(
        &mut self,
        mut exprs: Vec<Expression>,
    ) -> Result<Option<BasicValueEnum<'a>>, CompilationError> {
        if exprs.len() > 0 {
            let last = exprs.pop().unwrap();
            for expr in exprs {
                self.compile_ast(expr)?;
            }
            self.compile_ast(last)
        } else {
            Ok(None)
        }
    }
    fn compile_negative(
        &mut self,
        expr: Expression,
    ) -> Result<Option<BasicValueEnum<'a>>, CompilationError> {
        if let Expression::Negative(neg) = expr {
            return self.compile_ast(*neg);
        }
        let val = self.compile_ast(expr.clone())?.unwrap(); //no sense to be void
        if val.is_int_value() {
            Ok(Some(
                self.builder
                    .build_int_neg(val.into_int_value(), "intneg")
                    .unwrap()
                    .as_basic_value_enum(),
            ))
        } else if val.is_float_value() {
            Ok(Some(
                self.builder
                    .build_float_neg(val.into_float_value(), "floatneg")
                    .unwrap()
                    .as_basic_value_enum(),
            ))
        } else {
            Err(CompilationError::InvalidNegation(expr))
        }
    }
    fn compile_vardecl(
        &mut self,
        _kind: LetDeclKind,
        varname: &str,
        expr: Expression,
    ) -> Result<PointerValue<'a>, CompilationError> {
        let stype = self.type_based_on_semantics(&expr)?;
        let alloc = self
            .builder
            .build_alloca(
                if let Some(stype) = stype {
                    stype
                } else {
                    return Err(CompilationError::TryingAssignVoid);
                },
                varname,
            )
            .unwrap();
        if let Some(expr) = self.compile_ast(expr)? {
            self.builder.build_store(alloc, expr).unwrap();
        }
        self.variables.insert(varname.to_string(), alloc);
        Ok(alloc)
    }
    fn compile_binexpr(
        &mut self,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        operator: Operator,
        stype: SemanticType,
    ) -> Result<BasicValueEnum<'a>, CompilationError> {
        let lhs = self.compile_ast(*lhs)?.unwrap();
        let rhs = self.compile_ast(*rhs)?.unwrap();
        Ok(match stype {
            SemanticType::Int32 => {
                let lhs = lhs.into_int_value();
                let rhs = rhs.into_int_value();
                match operator {
                    Operator::Plus => self.builder.build_int_add(lhs, rhs, "addition").unwrap(),
                    Operator::Minus => self.builder.build_int_sub(lhs, rhs, "subtraction").unwrap(),
                    Operator::Star => self
                        .builder
                        .build_int_mul(lhs, rhs, "multiplication")
                        .unwrap(),
                    Operator::Bar => self
                        .builder
                        .build_int_signed_div(lhs, rhs, "division")
                        .unwrap(),
                    _ => panic!("{operator:?} is invalid or gotta be implemented"),
                }
                .as_basic_value_enum()
            }
            SemanticType::Float32 => {
                let lhs = lhs.into_float_value();
                let rhs = rhs.into_float_value();
                match operator {
                    Operator::Plus => self.builder.build_float_add(lhs, rhs, "addition").unwrap(),
                    Operator::Minus => self
                        .builder
                        .build_float_sub(lhs, rhs, "subtraction")
                        .unwrap(),
                    Operator::Star => self
                        .builder
                        .build_float_mul(lhs, rhs, "multiplication")
                        .unwrap(),
                    Operator::Bar => self.builder.build_float_div(lhs, rhs, "division").unwrap(),
                    _ => panic!("{operator:?} is invalid or gotta be implemented"),
                }
                .as_basic_value_enum()
            }
            t => {
                return Err(CompilationError::TypeError(SemanticError::InvalidBinExpr {
                    lhs_type: t.clone(),
                    rhs_type: t,
                }));
            }
        })
    }
}

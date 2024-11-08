use std::{collections::HashMap, path::Path};

use super::errors::CompilationError;
use crate::analysis::{errors::SemanticError, SemanticAnalayzer, SemanticType};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, FloatType, FunctionType, IntType, VoidType},
    values::{BasicValue, BasicValueEnum, FunctionValue, PointerValue},
    AddressSpace,
};
use parser::{
    parsing::{Expression, LetDeclKind, Param},
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

pub enum CodeGenType<'a> {
    Primitive(BasicTypeEnum<'a>),
    Fn(FunctionType<'a>),
}

impl<'a> CodeGenerator<'a> {
    pub fn create_ctx() -> Context {
        Context::create()
    }
    pub fn new(context: &'a Context) -> Self {
        let builder = context.create_builder();
        let module = context.create_module("hdc");
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
        let mut parser = parser::parsing::Parser::new();
        let ast = parser
            .parse_tokens(&mut tokens)
            .map_err(|e| CompilationError::Parsing(e, parser.backtrace))?;
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
    fn type_from_stype(&self, stype: &SemanticType) -> Option<CodeGenType<'a>> {
        Some(match stype {
            SemanticType::Int32 => CodeGenType::Primitive(self.i32().as_basic_type_enum()),
            SemanticType::Float32 => CodeGenType::Primitive(self.f32().as_basic_type_enum()),
            SemanticType::Void => return None,
            SemanticType::FnType { params, rtype } => {
                let params = {
                    let mut param_types = Vec::with_capacity(params.len());
                    for param in params {
                        if let Some(ptype) = self.type_from_stype(param) {
                            param_types.push(match ptype {
                                CodeGenType::Primitive(basic) => basic.into(),
                                CodeGenType::Fn(f) => f
                                    .ptr_type(AddressSpace::default())
                                    .as_basic_type_enum()
                                    .into(),
                            })
                        } else {
                            continue;
                        }
                    }
                    param_types
                };
                CodeGenType::Fn(if let Some(rtype) = self.type_from_stype(&**rtype) {
                    match rtype {
                        CodeGenType::Fn(f) => f
                            .ptr_type(AddressSpace::default())
                            .as_basic_type_enum()
                            .fn_type(&params, false),
                        CodeGenType::Primitive(f) => f.fn_type(&params, false),
                    }
                } else {
                    self.void().fn_type(&params, false)
                })
            }
        })
    }
    fn type_based_on_semantics(
        &mut self,
        expr: &Expression,
    ) -> Result<Option<CodeGenType<'a>>, CompilationError> {
        let stype = self
            .analyzer
            .analyze_expr(expr)
            .map_err(CompilationError::TypeError)?;
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
                ref identifier,
                ref block,
                ..
            } => {
                let stype = self
                    .analyzer
                    .analyze_expr(&expr)
                    .map_err(CompilationError::TypeError)?;
                Some(
                    self.compile_func_decl(identifier.clone(), block.clone(), stype)?
                        .as_global_value()
                        .as_basic_value_enum(),
                )
            }
        })
    }
    fn compile_func_decl(
        &mut self,
        identifier: String,
        block: Box<Expression>,
        stype: SemanticType,
    ) -> Result<FunctionValue<'a>, CompilationError> {
        let ftype = {
            self.analyzer
                .create_var(&identifier, &*block)
                .map_err(CompilationError::TypeError)?;

            let CodeGenType::Fn(func) = self.type_from_stype(&stype).unwrap() else {
                //i know that it will be a function type
                unreachable!();
            };
            func
        };
        let f = self.module.add_function(&identifier, ftype, None);
        let entry = self.context.append_basic_block(f, "entry");
        self.builder.position_at_end(entry);

        if self.variables.contains_key(&identifier) {
            return Err(CompilationError::InvalidRedeclare(identifier));
        } else {
            self.variables
                .insert(identifier, f.as_global_value().as_pointer_value());
        };
        match *block {
            Expression::Block(mut exprs) => {
                let last = exprs.pop().unwrap();
                for expr in exprs {
                    self.compile_ast(expr)?;
                }
                if let Some(expr) = self.compile_ast(last)? {
                    self.builder.build_return(Some(&expr)).unwrap();
                }
            }
            expr => {
                if let Some(expr) = self.compile_ast(expr)? {
                    self.builder.build_return(Some(&expr)).unwrap();
                }
            }
        };
        Ok(f)
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
                    match stype {
                        CodeGenType::Fn(f) => {
                            f.ptr_type(AddressSpace::default()).as_basic_type_enum()
                        }
                        CodeGenType::Primitive(basic) => basic,
                    }
                } else {
                    return Err(CompilationError::TryingAssignVoid);
                },
                varname,
            )
            .unwrap();
        let variable = varname.to_owned();
        self.analyzer
            .create_var(&variable, &expr)
            .map_err(CompilationError::TypeError)?;
        if let Some(expression) = self.compile_ast(expr)? {
            self.builder.build_store(alloc, expression).unwrap();
        } else {
            self.analyzer.delete_var(&variable);
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

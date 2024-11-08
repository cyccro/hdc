use std::{collections::HashMap, path::Path, str::FromStr};

use inkwell::{
    builder::{Builder, BuilderError},
    context::Context,
    module::Module,
    types::{BasicType, BasicTypeEnum, FloatType, IntType},
    values::{BasicValue, BasicValueEnum, PointerValue},
};
use parser::{
    parsing::{Expression, LetDeclKind},
    tokenizer::Operator,
};

use crate::analysis::{SemanticAnalayzer, SemanticType};

use super::errors::CompilationError;

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
    fn type_from_stype(&self, stype: &SemanticType) -> BasicTypeEnum<'a> {
        match stype {
            SemanticType::Int32 => self.i32().as_basic_type_enum(),
            SemanticType::Float32 => self.f32().as_basic_type_enum(),
        }
    }
    fn type_based_on_semantics(
        &mut self,
        expr: &Expression,
    ) -> Result<BasicTypeEnum<'a>, CompilationError> {
        let stype = self
            .analyzer
            .analyze_expr(expr)
            .map_err(|e| CompilationError::TypeError(e))?;
        Ok(self.type_from_stype(&stype))
    }
    fn compile_ast(&mut self, expr: Expression) -> Result<BasicValueEnum<'a>, CompilationError> {
        Ok(match expr {
            Expression::IntLit(s) => self
                .i32()
                .const_int(s.parse::<u64>().unwrap(), false)
                .as_basic_value_enum(),
            Expression::FloatLit(s) => self
                .f32()
                .const_float(s.parse::<f64>().unwrap())
                .as_basic_value_enum(),
            Expression::LetDecl {
                kind,
                varname,
                expr,
            } => self
                .compile_vardecl(kind, &varname, *expr)?
                .as_basic_value_enum(),
            Expression::Program(mut exprs) => {
                let last_expr = exprs.pop();
                for expr in exprs {
                    self.compile_ast(expr)?;
                }
                if let Some(last) = last_expr {
                    self.compile_ast(last)?
                } else {
                    self.i32().const_int(0, false).as_basic_value_enum()
                }
            }
            Expression::Identifier(s) => self.load(&s)?.as_basic_value_enum(),
            Expression::BinExpr { lhs, rhs, op } => {
                let stype = self
                    .analyzer
                    .analyze_binexpr(&lhs, &rhs)
                    .map_err(|e| CompilationError::TypeError(e))?;
                self.compile_binexpr(lhs, rhs, op, stype)?
            }
        })
    }
    fn compile_vardecl(
        &mut self,
        _kind: LetDeclKind,
        varname: &str,
        expr: Expression,
    ) -> Result<PointerValue<'a>, CompilationError> {
        let stype = self.type_based_on_semantics(&expr)?;
        let alloc = self.builder.build_alloca(stype, varname).unwrap();
        let expr = self.compile_ast(expr)?;
        self.builder.build_store(alloc, expr).unwrap();
        Ok(alloc)
    }
    fn compile_binexpr(
        &mut self,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        operator: Operator,
        stype: SemanticType,
    ) -> Result<BasicValueEnum<'a>, CompilationError> {
        let lhs = self.compile_ast(*lhs)?;
        let rhs = self.compile_ast(*rhs)?;
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
        })
    }
}

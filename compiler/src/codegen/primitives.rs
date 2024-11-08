use inkwell::{
    types::{BasicType, BasicTypeEnum},
    values::{BasicValue, BasicValueEnum},
};

use super::codegen::CodeGenerator;

pub trait InkWellType<'a>: Clone + Copy {
    fn itype(gen: &'a CodeGenerator) -> BasicTypeEnum<'a>;
    fn const_val(self, gen: &'a CodeGenerator) -> BasicValueEnum<'a>;
}

impl<'a> InkWellType<'a> for i32 {
    fn itype(gen: &'a CodeGenerator) -> BasicTypeEnum<'a> {
        gen.i32().as_basic_type_enum()
    }
    fn const_val(self, gen: &'a CodeGenerator) -> BasicValueEnum<'a> {
        gen.i32()
            .const_int(self as u64, false)
            .as_basic_value_enum()
    }
}
impl<'a> InkWellType<'a> for f32 {
    fn itype(gen: &'a CodeGenerator) -> BasicTypeEnum<'a> {
        gen.f32().as_basic_type_enum()
    }
    fn const_val(self, gen: &'a CodeGenerator) -> BasicValueEnum<'a> {
        gen.f32().const_float(self as f64).as_basic_value_enum()
    }
}

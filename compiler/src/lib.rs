pub mod analysis;
pub mod codegen;

use std::path::Path;

use crate::codegen::{codegen::CodeGenerator, errors::CompilationError};
pub fn compile_file(path: &Path) -> Result<Result<Vec<u8>, CompilationError>, std::io::Error> {
    let filecontent = std::fs::read_to_string(path)?;
    let ctx = CodeGenerator::create_ctx();
    let mut generator: CodeGenerator = CodeGenerator::new(&ctx);
    let output_path = {
        let mut split: Vec<String> = path
            .to_string_lossy()
            .split(".")
            .map(|e| e.to_string())
            .collect();
        split.pop();
        format!("{}.hdco", split.join("."))
    };
    let r = generator.compile_source(filecontent, Some(Path::new(&output_path)));
    Ok(r)
}
pub fn compile_from_to(
    input: &Path,
    output: &Path,
) -> Result<Result<Vec<u8>, CompilationError>, std::io::Error> {
    let filecontet = std::fs::read_to_string(input)?;
    let ctx = CodeGenerator::create_ctx();
    let mut generator: CodeGenerator = CodeGenerator::new(&ctx);
    let r = generator.compile_source(filecontet, Some(output));
    Ok(r)
}

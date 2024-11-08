use parser::{parsing::Parser, tokenizer::Tokenizer};
use std::{
    io::{Read, Write},
    path::Path,
};

fn get_file_content(path: &Path) -> Result<String, String> {
    if let Ok(mut f) = std::fs::File::open(path) {
        let mut buffer = String::new();
        f.read_to_string(&mut buffer).unwrap();
        Ok(buffer)
    } else {
        Err(format!(
            "Encountered some error while trying to get file at location: {path:?}"
        ))
    }
}

fn print_help() {
    println!("--List of Commands--");
    println!("hdc --help : shows this help list");
    println!("hdc <path> <optional>-o <path>: compiles the given file and if given -o <path>, creates the binary file in the given path, else, the same location of the hdc file")
}
fn print_err() {
    println!("Please use hdc --help to get help with commands");
}

fn main() {
    let env: Vec<String> = std::env::args().collect();
    if env.len() == 1 {
        return print_err();
    }
    let output = if let Some(o) = env.get(2) {
        if o == "-o" {
            env.get(3).expect("Expected a file output").clone()
        } else {
            format!("./{}.hdco", &env[1][..env[1].len() - 4])
        }
    } else {
        format!("./{}.hdco", &env[1][..env[1].len() - 4])
    };
    match &*env[1] {
        "hdc_help" => print_help(),
        _ => {
            match compiler::compile_from_to(Path::new(&env[1]), Path::new(&output)) {
                Err(e) => println!("{e:#?}"),
                Ok(r) => match r {
                    Ok(bytes) => println!("Bytes written:\n{bytes:?}"),
                    Err(e) => println!("Compilation Error: {e:#?}"),
                },
            };
        }
    }
}

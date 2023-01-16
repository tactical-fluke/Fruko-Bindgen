use clap::Parser;
use fruko_bindgen::compilation_target::CompilationTarget;
use fruko_bindgen::*;
use std::borrow::Borrow;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
struct Cli {
    /// Input data definition
    input_file: PathBuf,

    output_target: String,

    output_file: PathBuf,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let input_contents = std::fs::read_to_string(args.input_file)?;

    let tokens = lexer::lex_tokens(input_contents)?;
    let ast = parser::parse_tokens(tokens)?;

    let compilation_target = CompilationTarget::from_str(args.output_target.borrow())?;

    std::fs::write(args.output_file, compilation_target.generate_code(&ast))?;

    Ok(())
}

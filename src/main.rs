use clap::Parser;
use fruko_bindgen::compilation_target::Target;
use fruko_bindgen::*;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Parser)]
struct Cli {
    /// Input data definition file
    input_file: PathBuf,

    /// The files that the generated output will be placed into
    output_files: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    let input_contents = std::fs::read_to_string(args.input_file)?;

    let tokens = lexer::lex_tokens(input_contents)?;
    let ast = parser::parse_tokens(tokens)?;

    for file in args.output_files {
        let compilation_target = Target::from_str(file.extension().expect("should have extension").to_str().expect("should be valid UTF8"))?;

        std::fs::write(file, compilation_target.generate_code(&ast)?)?;
    }

    Ok(())
}

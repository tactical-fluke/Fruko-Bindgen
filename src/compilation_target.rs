use crate::parser::ASTNode;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;
use crate::cxx::CXXGenerator;
use crate::ts_mobx::TSMobXGenerator;

pub trait CompilationTarget {
    fn generate_code(&self, ast: &ASTNode) -> Result<String, CompilationError>;
}

pub struct Target {
    target: Box<dyn CompilationTarget>,
}

#[derive(Debug)]
pub enum CompilationError {
    UnknownTarget(String),
    InvalidAST,
}

impl Display for CompilationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationError::UnknownTarget(name) => {
                write!(f, "Unknown compilation target: {}", name)
            },
            Self::InvalidAST => write!(f, "Invalid AST to generator")
        }
    }
}

impl Error for CompilationError {}

impl FromStr for Target {
    type Err = CompilationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cxx" | "cpp" | "c++" | "h" => Ok(Self { target: Box::new(CXXGenerator {}) }),
            "ts-mobx" | "typescript-mobx" | "ts" => Ok(Self { target: Box::new(TSMobXGenerator {}) }),
            unknown_target => Err(CompilationError::UnknownTarget(
                unknown_target.to_owned(),
            )),
        }
    }
}

impl Target {
    pub fn generate_code(&self, ast: &ASTNode) -> Result<String, CompilationError> {
        self.target.generate_code(ast)
    }
}

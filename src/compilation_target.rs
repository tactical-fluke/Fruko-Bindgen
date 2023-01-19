use crate::parser::ASTNode;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub enum CompilationTarget {
    CXX,
    TsMobx,
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

impl FromStr for CompilationTarget {
    type Err = CompilationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cxx" | "cpp" | "c++" => Ok(Self::CXX),
            "ts-mobx" | "typescript-mobx" => Ok(Self::TsMobx),
            unknown_target => Err(CompilationError::UnknownTarget(
                unknown_target.to_owned(),
            )),
        }
    }
}

impl CompilationTarget {
    pub fn generate_code(&self, ast: &ASTNode) -> Result<String, CompilationError> {
        match self {
            CompilationTarget::CXX => crate::cxx::generate_code(ast),
            CompilationTarget::TsMobx => crate::ts_mobx::generate_code(ast),
        }
    }
}

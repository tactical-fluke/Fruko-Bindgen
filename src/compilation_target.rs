use crate::parser::ASTNode;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

pub enum CompilationTarget {
    CXX,
    TsMobx,
}

#[derive(Debug)]
pub enum CompilationTargetError {
    UnknownTarget(String),
}

impl Display for CompilationTargetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CompilationTargetError::UnknownTarget(name) => {
                write!(f, "Unknown compilation target: {}", name)
            }
        }
    }
}

impl Error for CompilationTargetError {}

impl FromStr for CompilationTarget {
    type Err = CompilationTargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cxx" | "cpp" | "c++" => Ok(Self::CXX),
            "ts-mobx" | "typescript-mobx" => Ok(Self::TsMobx),
            unknown_target => Err(CompilationTargetError::UnknownTarget(
                unknown_target.to_owned(),
            )),
        }
    }
}

impl CompilationTarget {
    pub fn generate_code(&self, ast: &ASTNode) -> String {
        match self {
            CompilationTarget::CXX => crate::cxx::generate_code(ast),
            CompilationTarget::TsMobx => crate::ts_mobx::generate_code(ast),
        }
    }
}

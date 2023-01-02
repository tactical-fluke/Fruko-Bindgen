use crate::parser::ASTNode;
use std::str::FromStr;

pub enum CompilationTarget {
    CXX,
}

pub enum CompilationTargetError {
    UnknownTarget,
}

impl FromStr for CompilationTarget {
    type Err = CompilationTargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cxx" | "cpp" | "c++" => Ok(Self::CXX),
            _ => Err(CompilationTargetError::UnknownTarget),
        }
    }
}

impl CompilationTarget {
    pub fn generate_code(&self, ast: ASTNode) -> String {
        match self {
            CompilationTarget::CXX => crate::cxx::generate_code(ast),
        }
    }
}

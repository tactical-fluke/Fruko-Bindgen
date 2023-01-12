use crate::parser::ASTNode;
use std::str::FromStr;

pub enum CompilationTarget {
    CXX,
    TsMobx,
}

pub enum CompilationTargetError {
    UnknownTarget,
}

impl FromStr for CompilationTarget {
    type Err = CompilationTargetError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cxx" | "cpp" | "c++" => Ok(Self::CXX),
            "ts-mobx" | "typescript-mobx" => Ok(Self::TsMobx),
            _ => Err(CompilationTargetError::UnknownTarget),
        }
    }
}

impl CompilationTarget {
    pub fn generate_code(&self, ast: ASTNode) -> String {
        match self {
            CompilationTarget::CXX => crate::cxx::generate_code(ast),
            CompilationTarget::TsMobx => crate::ts_mobx::generate_code(&ast),
        }
    }
}

///Code generation for C++
/// Includes AST transformation, which effectively just pulls inline struct and enum declarations
/// out of line
use crate::parser::{
    ASTNode, DataDefinition, DataType, NamedStatementList, StructMemberDeclaration,
};
use std::borrow::Borrow;

/// Main C++ generation function. Takes an AST, and returns the generated code
/// # Parameters
/// ast - The abstract syntax tree of which to generate the code. It is assumed to be a valid data definition AST
/// # Return
/// returns the generated C++ cpde
pub fn generate_code(ast: &ASTNode) -> String {
    let new_ast = CXXASTTransformer::transform_ast(ast);
    generate(&new_ast)
}

/// Helper struct, made to just keep the transformed AST in memory whilst the function recursively
/// transforms it
struct CXXASTTransformer {
    new_ast: DataDefinition,
}

impl CXXASTTransformer {
    /// Main interface for the CXXASTTransformer
    fn transform_ast(ast: &ASTNode) -> ASTNode {
        let mut transformer = Self {
            new_ast: DataDefinition::default(),
        };
        transformer.transform_ast_impl(ast);
        ASTNode::DataDefinition(transformer.new_ast)
    }

    /// Does the actual transformation. The transformation is only taking inline struct and enum
    /// declarations out of line, the AST is otherwise untouched
    fn transform_ast_impl(&mut self, ast: &ASTNode) {
        match ast {
            ASTNode::StructDeclaration(struct_declaration) => {
                let mut pushed_struct = NamedStatementList::new(struct_declaration.name.clone());
                for member in &struct_declaration.child_nodes {
                    if let ASTNode::StructMemberDeclaration(member_declaration) = member {
                        match member_declaration.data_type.borrow() {
                            ASTNode::StructDeclaration(inline_struct_declaration) => {
                                self.transform_ast_impl(member_declaration.data_type.borrow());
                                let new_member = StructMemberDeclaration {
                                    name: member_declaration.name.clone(),
                                    data_type: Box::new(ASTNode::TypeLiteral(
                                        DataType::UserDefined(
                                            inline_struct_declaration.name.clone(),
                                        ),
                                    )),
                                };
                                pushed_struct
                                    .child_nodes
                                    .push(ASTNode::StructMemberDeclaration(new_member));
                            }
                            ASTNode::EnumDeclaration(inline_enum_declaration) => {
                                self.transform_ast_impl(member_declaration.data_type.borrow());
                                let new_member = StructMemberDeclaration {
                                    name: member_declaration.name.clone(),
                                    data_type: Box::new(ASTNode::TypeLiteral(
                                        DataType::UserDefined(inline_enum_declaration.name.clone()),
                                    )),
                                };
                                pushed_struct
                                    .child_nodes
                                    .push(ASTNode::StructMemberDeclaration(new_member));
                            }
                            _ => pushed_struct.child_nodes.push(member.clone()),
                        }
                    }
                }
                self.new_ast
                    .child_nodes
                    .push(ASTNode::StructDeclaration(pushed_struct));
            }
            ASTNode::EnumDeclaration(enum_declaration) => {
                self.new_ast
                    .child_nodes
                    .push(ASTNode::EnumDeclaration(enum_declaration.clone()));
            }
            ASTNode::DataDefinition(data) => {
                for child in &data.child_nodes {
                    self.transform_ast_impl(child);
                }
            }
            _ => {}
        }
    }
}

/// Turns the transformed AST into a string of valid C++
fn generate(ast: &ASTNode) -> String {
    match ast {
        ASTNode::StructDeclaration(struct_definition) => {
            let body = struct_definition
                .child_nodes
                .iter()
                .map(generate)
                .fold(String::new(), |acc, x| acc + &x);
            format!("struct {} {{ {} }};", struct_definition.name, body)
        }
        ASTNode::EnumDeclaration(enum_declaration) => {
            let body = enum_declaration
                .child_nodes
                .iter()
                .map(generate)
                .collect::<Vec<String>>() // Iterator::intersperse is unstable
                .join(",");
            format!("enum class {} {{ {} }};", enum_declaration.name, body)
        }
        ASTNode::StructMemberDeclaration(member) => {
            format!("{} {};", generate(member.data_type.borrow()), member.name)
        }
        ASTNode::EnumMemberDeclaration(member) => member.name.clone(),
        ASTNode::TypeLiteral(type_name) => generate_type_name(type_name),
        ASTNode::DataDefinition(def) => def
            .child_nodes
            .iter()
            .map(generate)
            .fold(String::new(), |acc, x| acc + &x),
    }
}

/// Generates the type name for the supplied data type. I have used the standard fixed width numeric
/// types simply for ease of use here.
///
/// For the optional and array types, the inner types are generated recursively
fn generate_type_name(data_type: &DataType) -> String {
    match data_type {
        DataType::U8 => "std::uint8_t".to_owned(),
        DataType::I8 => "std::int8_t".to_owned(),
        DataType::U16 => "std::uint16_t".to_owned(),
        DataType::I16 => "std::int16_t".to_owned(),
        DataType::U32 => "std::uint32_t".to_owned(),
        DataType::I32 => "std::int32_t".to_owned(),
        DataType::U64 => "std::uint64_t".to_owned(),
        DataType::I64 => "std::int64_t".to_owned(),
        DataType::F32 => "float".to_owned(),
        DataType::F64 => "double".to_owned(),
        DataType::Char => "char".to_owned(),
        DataType::String => "std::string".to_owned(),
        DataType::Bool => "bool".to_owned(),
        DataType::Option(inner_type) => {
            format!("std::optional<{}>", generate_type_name(inner_type))
        }
        DataType::Array(inner_type) => {
            format!("std::vector<{}>", generate_type_name(inner_type))
        }
        DataType::UserDefined(name) => name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED_OUTPUT: &str = "struct inner struct 2 {  };struct inner struct 1 { inner struct 2 inner struct 1 member 1; };struct outer struct { inner struct 1 outer struct member 1; };";

    fn initial_ast() -> ASTNode {
        ASTNode::DataDefinition(DataDefinition {
            child_nodes: vec![ASTNode::StructDeclaration(NamedStatementList {
                name: "outer struct".to_owned(),
                child_nodes: vec![ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                    name: "outer struct member 1".to_owned(),
                    data_type: Box::new(ASTNode::StructDeclaration(NamedStatementList {
                        name: "inner struct 1".to_owned(),
                        child_nodes: vec![ASTNode::StructMemberDeclaration(
                            StructMemberDeclaration {
                                name: "inner struct 1 member 1".to_owned(),
                                data_type: Box::new(ASTNode::StructDeclaration(
                                    NamedStatementList {
                                        name: "inner struct 2".to_owned(),
                                        child_nodes: Vec::new(),
                                    },
                                )),
                            },
                        )],
                    })),
                })],
            })],
        })
    }

    fn transformed_ast() -> ASTNode {
        ASTNode::DataDefinition(DataDefinition {
            child_nodes: vec![
                ASTNode::StructDeclaration(NamedStatementList {
                    name: "inner struct 2".to_owned(),
                    child_nodes: Vec::new(),
                }),
                ASTNode::StructDeclaration(NamedStatementList {
                    name: "inner struct 1".to_owned(),
                    child_nodes: vec![ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: "inner struct 1 member 1".to_owned(),
                        data_type: Box::new(ASTNode::TypeLiteral(DataType::UserDefined(
                            "inner struct 2".to_owned(),
                        ))),
                    })],
                }),
                ASTNode::StructDeclaration(NamedStatementList {
                    name: "outer struct".to_owned(),
                    child_nodes: vec![ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: "outer struct member 1".to_owned(),
                        data_type: Box::new(ASTNode::TypeLiteral(DataType::UserDefined(
                            "inner struct 1".to_owned(),
                        ))),
                    })],
                }),
            ],
        })
    }

    #[test]
    fn test_ast_transformation() {
        assert_eq!(
            CXXASTTransformer::transform_ast(&initial_ast()),
            transformed_ast()
        );
    }

    #[test]
    fn test_cxx_code_generation() {
        assert_eq!(generate(&transformed_ast()), GENERATED_OUTPUT);
    }

    #[test]
    fn test_cxx_generation() {
        assert_eq!(generate_code(&initial_ast()), GENERATED_OUTPUT);
    }
}

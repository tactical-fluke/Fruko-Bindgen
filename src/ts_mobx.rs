/// Generation for Typescript with MobX
///
/// The AST is not transformed at all, as the declaration structure is the same as the allowed
/// data definition structure
///
/// The root level data definitions come with some slight extras, exporting both the regular
/// type using the MobX types, as well as a snapshot type, derived from the regular type.
///
/// NOTE: currently, using a non-inline enum definition may not work as expected.
use crate::compilation_target::{CompilationError, CompilationInfo, CompilationTarget};
use crate::parser::{ASTNode, DataType};
use std::borrow::Borrow;

pub struct TSMobXGenerator;

impl CompilationTarget for TSMobXGenerator {
    fn generate_code(&self, ast: &ASTNode, compilation_info: &CompilationInfo) -> Result<String, CompilationError> {
        Ok(format!("{}\n{}", generate_preamble(compilation_info), generate_code(ast)?))
    }
}

fn generate_preamble(compilation_info: &CompilationInfo) -> String {
    compilation_info.preamble_comments.iter()
        .map(|preamble| format!("// {}", preamble))
        .collect::<Vec<String>>()
        .join("\n")
}

/// Entry API for Typescript MobX code generation
fn generate_code(ast: &ASTNode) -> Result<String, CompilationError> {
    Ok(match ast {
        ASTNode::StructDeclaration(struct_declaration) => format!(
            "types.model({{ {} }})",
            struct_declaration
                .child_nodes
                .iter()
                .try_fold("".to_owned(), |acc, node| Ok(acc
                    + &generate_code(node)?
                    + ", "))?
        ),
        ASTNode::EnumDeclaration(enum_declaration) => format!(
            "types.enum([ {} ])",
            enum_declaration
                .child_nodes
                .iter()
                .try_fold("".to_owned(), |acc, node| Ok(acc
                    + &generate_code(node)?
                    + ", "))?
        ),
        ASTNode::StructMemberDeclaration(struct_member) => format!(
            "{}: {}",
            struct_member.name,
            generate_code(struct_member.data_type.borrow())?
        ),
        ASTNode::EnumMemberDeclaration(enum_member) => format!("'{}'", enum_member.name.clone()),
        ASTNode::TypeLiteral(data_type) => generate_type_name(data_type),
        ASTNode::DataDefinition(def) => def
            .child_nodes
            .iter()
            .try_fold("".to_owned(), |acc, node| {
                Ok(acc + &generate_top_level_type_definition(node)?)
            })?,
    })
}

/// Generates the specific top level exports
fn generate_top_level_type_definition(ast: &ASTNode) -> Result<String, CompilationError> {
    match ast {
        ASTNode::StructDeclaration(struct_declaration) => Ok(format!(
            "export const {} = {}; export type {}SnapshotType = SnapshotIn<typeof {}>;",
            struct_declaration.name,
            generate_code(ast)?,
            struct_declaration.name,
            struct_declaration.name
        )),
        ASTNode::EnumDeclaration(enum_declaration) => Ok(format!(
            "export const {} = {}; export type {}SnapshotType = SnapshotIn<typeof {}>;",
            enum_declaration.name,
            generate_code(ast)?,
            enum_declaration.name,
            enum_declaration.name
        )),
        _ => Err(CompilationError::InvalidAST),
    }
}

/// Generates the type names
fn generate_type_name(type_name: &DataType) -> String {
    match type_name {
        DataType::U8
        | DataType::I8
        | DataType::U16
        | DataType::I16
        | DataType::U32
        | DataType::I32
        | DataType::U64
        | DataType::I64
        | DataType::F32
        | DataType::F64 => "types.num".to_owned(),
        DataType::Char => "types.char".to_owned(),
        DataType::String => "types.string".to_owned(),
        DataType::Bool => "types.bool".to_owned(),
        DataType::Option(inner_type) => format!("types.maybe({})", generate_type_name(inner_type)),
        DataType::Array(inner_type) => format!("types.array({})", generate_type_name(inner_type)),
        DataType::UserDefined(name) => name.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DataDefinition, NamedStatementList, StructMemberDeclaration};

    fn initial_ast() -> ASTNode {
        ASTNode::DataDefinition(DataDefinition {
            child_nodes: vec![ASTNode::StructDeclaration(NamedStatementList {
                name: "struct1".to_owned(),
                child_nodes: vec![
                    ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: "member1".to_owned(),
                        data_type: Box::new(ASTNode::TypeLiteral(DataType::U32)),
                    }),
                    ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: "member2".to_owned(),
                        data_type: Box::new(ASTNode::TypeLiteral(DataType::F64)),
                    }),
                    ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: "member3".to_owned(),
                        data_type: Box::new(ASTNode::TypeLiteral(DataType::Option(Box::new(
                            DataType::String,
                        )))),
                    }),
                    ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: "member4".to_owned(),
                        data_type: Box::new(ASTNode::StructDeclaration(NamedStatementList {
                            name: "inner_struct".to_owned(),
                            child_nodes: vec![ASTNode::StructMemberDeclaration(
                                StructMemberDeclaration {
                                    name: "member1".to_owned(),
                                    data_type: Box::new(ASTNode::TypeLiteral(DataType::Bool)),
                                },
                            )],
                        })),
                    }),
                ],
            })],
        })
    }

    const GENERATED_CODE: &str = "export const struct1 = types.model({ member1: types.num, member2: types.num, member3: types.maybe(types.string), member4: types.model({ member1: types.bool,  }),  }); export type struct1SnapshotType = SnapshotIn<typeof struct1>;";

    #[test]
    fn test_generate_ts_mobx() {
        assert_eq!(
            generate_code(&initial_ast()).expect("should generate code"),
            GENERATED_CODE.to_owned()
        )
    }
}

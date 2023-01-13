use crate::lexer::{Token, TokenList, TokenType};
use std::iter::Peekable;

/// Node type for our Abstract Syntax Tree (AST)
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum ASTNode {
    StructDeclaration(NamedStatementList),
    EnumDeclaration(NamedStatementList),
    StructMemberDeclaration(StructMemberDeclaration),
    EnumMemberDeclaration(EnumMemberDeclaration),
    TypeLiteral(DataType),
    DataDefinition(DataDefinition),
}

/// All of our supported data types, including types defined by the user (e.g a struct or enum).
/// During parsing, we do not check whether a user defined type has actually been defined
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DataType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F32,
    F64,
    Char,
    String,
    Bool,
    Option(Box<DataType>),
    Array(Box<DataType>),
    UserDefined(String),
}

/// Data required to define a struct member
/// data_type takes an ASTNode to allow inline definition of a struct or enum
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct StructMemberDeclaration {
    pub name: String,
    pub data_type: Box<ASTNode>,
}

/// Simple enum member declaration; Only has a name (we do not emulate an underlying type)
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct EnumMemberDeclaration {
    pub name: String,
}

/// A named statement list - Either a struct or an enum.
#[derive(PartialEq, Eq, Debug, Clone)]
pub struct NamedStatementList {
    pub name: String,
    pub child_nodes: Vec<ASTNode>,
}

impl NamedStatementList {
    pub fn new(name: String) -> Self {
        Self {
            name,
            child_nodes: Vec::new(),
        }
    }
}

/// Root node for our AST
#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct DataDefinition {
    pub child_nodes: Vec<ASTNode>,
}

/// Simple parsing error type
/// # Meanings
/// UnexpectedToken - an out of place token was found while parsing,
/// UnexpectedEndOfTokens - The tokens ended before an ASTNode was finished parsing
#[derive(Debug, Clone)]
pub enum ParseError {
    UnexpectedToken,
    UnexpectedEndOfTokens,
}

/// Entry parsing function
/// # Parameters
/// tokens - The Tokenlist from which to derive our AST from
/// # Returns
/// An ASTNode::DataDefinition if the tokens were parsed successfully into an AST, otherwise
/// a ParseError
pub fn parse_tokens(tokens: TokenList) -> Result<ASTNode, ParseError> {
    let mut base_ast_node = DataDefinition::default();
    let mut iterator = tokens.0.iter().peekable();

    while iterator.peek() != None {
        base_ast_node.child_nodes.push(parse(&mut iterator)?);
    }

    Ok(ASTNode::DataDefinition(base_ast_node))
}

/// Helper function - unwraps a result of a token iterator peek, or returns an UnexpectedEndOfTokens
/// error
fn unwrap_peek_or_error<'a>(token: Option<&&'a Token>) -> Result<&'a Token, ParseError> {
    match token {
        Some(tok) => Ok(tok),
        None => Err(ParseError::UnexpectedEndOfTokens),
    }
}

/// Helper function - unwraps a result of a token iterator next call, or returns an
/// UnexpectedEndOfTokens error
fn unwrap_or_error(token: Option<&Token>) -> Result<&Token, ParseError> {
    match token {
        Some(tok) => Ok(tok),
        None => Err(ParseError::UnexpectedEndOfTokens),
    }
}

/// Asserts that the supplied token is the one expected.
/// # Parameters
/// token - The token to check. Normally used here as the result of a call to token_iter.next()
/// expected_token: The expected token type
/// # Returns
/// () if the token was both Some, and the type was the expected type,
/// ParseError::UnexpectedToken if the token was not the expected type,
/// ParseError::UnexpectedEndOfTokens if token was None
fn assert_token(token: Option<&Token>, expected_token: TokenType) -> Result<(), ParseError> {
    match token {
        Some(token) => {
            if token.token_type == expected_token {
                Ok(())
            } else {
                Err(ParseError::UnexpectedToken)
            }
        }
        None => Err(ParseError::UnexpectedEndOfTokens),
    }
}

/// Main work function for parsing our AST. Deals with the top level of our data definition,
/// covering our raw enum and struct definitions
fn parse<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<ASTNode, ParseError> {
    let token = unwrap_or_error(token_iter.next())?;

    match &token.token_type {
        TokenType::Struct => Ok(ASTNode::StructDeclaration(parse_named_statement_list(
            token_iter,
        )?)),
        TokenType::Enum => Ok(ASTNode::EnumDeclaration(parse_named_statement_list(
            token_iter,
        )?)),
        _ => Err(ParseError::UnexpectedToken),
    }
}

/// Parses the body of a named statement list. This could either be a struct or an enum.
fn parse_named_statement_list<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<NamedStatementList, ParseError> {
    let name_token = unwrap_or_error(token_iter.next())?;
    let name = match &name_token.token_type {
        TokenType::Identifier(name) => name,
        _ => return Err(ParseError::UnexpectedToken),
    };

    assert_token(token_iter.next(), TokenType::LCurly)?;

    let named_statement_list = NamedStatementList {
        name: name.clone(),
        child_nodes: parse_named_statement_list_children(token_iter)?,
    };

    assert_token(token_iter.next(), TokenType::RCurly)?;

    Ok(named_statement_list)
}

/// Parses the child nodes of a named statement list.
/// Note that the child nodes could be either a struct member declaration, or an enum member
/// declaration, without taking into account what the named statement list actually is. This must
/// be verified at a later stage
fn parse_named_statement_list_children<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<Vec<ASTNode>, ParseError> {
    let mut ret_val = Vec::new();
    loop {
        let name_token = unwrap_peek_or_error(token_iter.peek())?;
        let name = match &name_token.token_type {
            TokenType::Identifier(name) => name,
            _ => break,
        };
        token_iter.next();

        let following_token = unwrap_or_error(token_iter.next())?;

        ret_val.push(match following_token.token_type {
            TokenType::Colon => {
                let struct_member_decl = StructMemberDeclaration {
                    name: name.clone(),
                    data_type: Box::new(parse_struct_member_type_declaration(token_iter)?),
                };
                ASTNode::StructMemberDeclaration(struct_member_decl)
            }
            TokenType::Comma => {
                ASTNode::EnumMemberDeclaration(EnumMemberDeclaration { name: name.clone() })
            }
            _ => return Err(ParseError::UnexpectedToken),
        });

        let potential_comma = unwrap_peek_or_error(token_iter.peek())?;
        if let TokenType::Comma = potential_comma.token_type {
            token_iter.next(); // Iterate over the comma
        }
    }
    Ok(ret_val)
}

/// Parses a struct member type declaration. This could either by a TypeLiteral, an inline struct
/// definition, or an inline enum definition
fn parse_struct_member_type_declaration<'a>(
    token_iter: &mut Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<ASTNode, ParseError> {
    let token = unwrap_or_error(token_iter.next())?;

    match &token.token_type {
        TokenType::Struct => Ok(ASTNode::StructDeclaration(parse_named_statement_list(
            token_iter,
        )?)),
        TokenType::Enum => Ok(ASTNode::EnumDeclaration(parse_named_statement_list(
            token_iter,
        )?)),
        type_token => Ok(ASTNode::TypeLiteral(parse_literal_type(
            type_token, token_iter,
        )?)),
    }
}

/// Parses a type literal. In the case of an array or an option, the inner types are parsed
/// recursively
fn parse_literal_type<'a>(
    type_token: &TokenType,
    token_iter: &mut Peekable<impl Iterator<Item = &'a Token>>,
) -> Result<DataType, ParseError> {
    match type_token {
        TokenType::U8 => Ok(DataType::U8),
        TokenType::U16 => Ok(DataType::U16),
        TokenType::U32 => Ok(DataType::U32),
        TokenType::U64 => Ok(DataType::U64),
        TokenType::I8 => Ok(DataType::I8),
        TokenType::I16 => Ok(DataType::I16),
        TokenType::I32 => Ok(DataType::I32),
        TokenType::I64 => Ok(DataType::I64),
        TokenType::F32 => Ok(DataType::F32),
        TokenType::F64 => Ok(DataType::F64),
        TokenType::String => Ok(DataType::String),
        TokenType::Char => Ok(DataType::Char),
        TokenType::Bool => Ok(DataType::Bool),
        TokenType::Option => {
            assert_token(token_iter.next(), TokenType::LParen)?;
            let data_type = DataType::Option(Box::new(parse_literal_type(
                &unwrap_or_error(token_iter.next())?.token_type,
                token_iter,
            )?));
            assert_token(token_iter.next(), TokenType::RParen)?;
            Ok(data_type)
        }
        TokenType::Array => {
            assert_token(token_iter.next(), TokenType::LParen)?;
            let data_type = DataType::Option(Box::new(parse_literal_type(
                &unwrap_or_error(token_iter.next())?.token_type,
                token_iter,
            )?));
            assert_token(token_iter.next(), TokenType::RParen)?;
            Ok(data_type)
        }
        TokenType::Identifier(name) => Ok(DataType::UserDefined(name.clone())),
        _ => Err(ParseError::UnexpectedToken),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "struct name { member1: u32, member2: option(f32) }";

    #[test]
    fn test_correctly_parses_ast() {
        use crate::lexer::lex_tokens;
        let tokens = lex_tokens(String::from(TEXT)).expect("should be able to lex");

        let ast_start = parse_tokens(tokens).expect("should be able to parse");
        let expected_ast = ASTNode::DataDefinition(DataDefinition {
            child_nodes: vec![ASTNode::StructDeclaration(NamedStatementList {
                name: String::from("name"),
                child_nodes: vec![
                    ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: String::from("member1"),
                        data_type: Box::new(ASTNode::TypeLiteral(DataType::U32)),
                    }),
                    ASTNode::StructMemberDeclaration(StructMemberDeclaration {
                        name: String::from("member2"),
                        data_type: Box::new(ASTNode::TypeLiteral(DataType::Option(Box::new(
                            DataType::F32,
                        )))),
                    }),
                ],
            })],
        });

        assert_eq!(ast_start, expected_ast);
    }
}

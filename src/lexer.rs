use std::error::Error;
use std::fmt::{Display, Formatter};
use std::iter::Peekable;
use std::str::Chars;

/// Represents a location within a string
/// Which line, and at what position that line this thing is on
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SourceLocation {
    pub line: i32,
    pub position: i32,
}

impl Default for SourceLocation {
    fn default() -> Self {
        Self {
            line: 1,
            position: 0,
        }
    }
}

impl Display for SourceLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.position)
    }
}

/// Our token type
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum TokenType {
    // Grammar
    LParen,
    RParen,
    LCurly,
    RCurly,
    LSquare,
    RSquare,
    Comma,
    Colon,
    // keywords
    Struct,
    Enum,
    // type keywords
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    String,
    Char,
    Bool,
    Option,
    Array,
    // Generic string identifier
    Identifier(String),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub source_location: SourceLocation,
}

/// Simple  error type
#[derive(Debug, PartialEq, Eq)]
pub enum LexError {
    UnknownCharacterError(SourceLocation),
}

impl Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnknownCharacterError(location) => {
                write!(f, "Unknown character at {}", location)
            }
        }
    }
}

impl Error for LexError {}

#[derive(Debug, PartialEq, Eq)]
pub struct TokenList(pub Vec<Token>);

/// Transforms a string into a list of Tokens
/// This is intended to make parsing much easier
/// Whitespace is disregarded and discarded, and so has no effect on the tokens
/// # Arguments
/// `contents` - The string to extract tokens from
/// # Returns
/// OK(TokenList) if the string is parsed without error
/// Err(UnknownCharacterError) if an unknown character is encountered
pub fn lex_tokens(contents: String) -> Result<TokenList, LexError> {
    Lexer {
        source_location: SourceLocation::default(),
        iterator: contents.chars().into_iter().peekable(),
    }
    .lex_impl()
}

/// Helper struct to allow for tracking the source location of tokens
struct Lexer<'a> {
    source_location: SourceLocation,
    iterator: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    fn peek(&mut self) -> Option<&char> {
        self.iterator.peek()
    }

    fn next(&mut self) -> Option<char> {
        let char = self.iterator.next();
        if let Some(character) = &char {
            self.source_location.position += 1;
            if character == &'\n' {
                self.source_location.line += 1;
                self.source_location.position = 0;
            }
        }

        char
    }

    /// Does the actual lexing
    pub fn lex_impl(&mut self) -> Result<TokenList, LexError> {
        let mut tokens = Vec::new();

        while self.peek() != None {
            let char = self.next().unwrap();

            if char.is_whitespace() {
                continue;
            }

            let source_location = self.source_location.clone();

            tokens.push(match char {
                '(' => Token {
                    token_type: TokenType::LParen,
                    source_location,
                },
                '{' => Token {
                    token_type: TokenType::LCurly,
                    source_location,
                },
                '[' => Token {
                    token_type: TokenType::LSquare,
                    source_location,
                },
                ')' => Token {
                    token_type: TokenType::RParen,
                    source_location,
                },
                '}' => Token {
                    token_type: TokenType::RCurly,
                    source_location,
                },
                ']' => Token {
                    token_type: TokenType::RSquare,
                    source_location,
                },
                ',' => Token {
                    token_type: TokenType::Comma,
                    source_location,
                },
                ':' => Token {
                    token_type: TokenType::Colon,
                    source_location,
                },
                x if x.is_alphanumeric() => self.lex_name(x),
                _ => return Err(LexError::UnknownCharacterError(source_location)),
            });
        }

        Ok(TokenList(tokens))
    }

    /// Lexes a name, being any of a struct, enum, or named identifier
    fn lex_name(&mut self, start_char: char) -> Token {
        let mut name = String::from(start_char);
        let source_location = self.source_location.clone();

        while self.peek() != None && self.peek().unwrap().is_alphanumeric() {
            name.push(self.next().unwrap());
        }

        match name.as_str() {
            "struct" => Token {
                token_type: TokenType::Struct,
                source_location,
            },
            "enum" => Token {
                token_type: TokenType::Enum,
                source_location,
            },
            "u8" => Token {
                token_type: TokenType::U8,
                source_location,
            },
            "u16" => Token {
                token_type: TokenType::U16,
                source_location,
            },
            "u32" => Token {
                token_type: TokenType::U32,
                source_location,
            },
            "u64" => Token {
                token_type: TokenType::U64,
                source_location,
            },
            "i8" => Token {
                token_type: TokenType::I8,
                source_location,
            },
            "i16" => Token {
                token_type: TokenType::I16,
                source_location,
            },
            "i32" => Token {
                token_type: TokenType::I32,
                source_location,
            },
            "i64" => Token {
                token_type: TokenType::I64,
                source_location,
            },
            "f32" => Token {
                token_type: TokenType::F32,
                source_location,
            },
            "f64" => Token {
                token_type: TokenType::F64,
                source_location,
            },
            "string" => Token {
                token_type: TokenType::String,
                source_location,
            },
            "char" => Token {
                token_type: TokenType::Char,
                source_location,
            },
            "bool" => Token {
                token_type: TokenType::Bool,
                source_location,
            },
            "option" => Token {
                token_type: TokenType::Option,
                source_location,
            },
            "array" => Token {
                token_type: TokenType::Array,
                source_location,
            },
            _ => Token {
                token_type: TokenType::Identifier(name),
                source_location,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    const TEST_DATA: &str = "{ name: string, age: u32, height: f32 }";

    use super::*;

    #[test]
    fn test_expected_tokens() {
        let tokens = lex_tokens(String::from(TEST_DATA)).expect("should be able to tokenize");
        let expected_tokens = vec![
            Token {
                token_type: TokenType::LCurly,
                source_location: SourceLocation {
                    line: 1,
                    position: 1,
                },
            },
            Token {
                token_type: TokenType::Identifier(String::from("name")),
                source_location: SourceLocation {
                    line: 1,
                    position: 3,
                },
            },
            Token {
                token_type: TokenType::Colon,
                source_location: SourceLocation {
                    line: 1,
                    position: 7,
                },
            },
            Token {
                token_type: TokenType::String,
                source_location: SourceLocation {
                    line: 1,
                    position: 9,
                },
            },
            Token {
                token_type: TokenType::Comma,
                source_location: SourceLocation {
                    line: 1,
                    position: 15,
                },
            },
            Token {
                token_type: TokenType::Identifier(String::from("age")),
                source_location: SourceLocation {
                    line: 1,
                    position: 17,
                },
            },
            Token {
                token_type: TokenType::Colon,
                source_location: SourceLocation {
                    line: 1,
                    position: 20,
                },
            },
            Token {
                token_type: TokenType::U32,
                source_location: SourceLocation {
                    line: 1,
                    position: 22,
                },
            },
            Token {
                token_type: TokenType::Comma,
                source_location: SourceLocation {
                    line: 1,
                    position: 25,
                },
            },
            Token {
                token_type: TokenType::Identifier(String::from("height")),
                source_location: SourceLocation {
                    line: 1,
                    position: 27,
                },
            },
            Token {
                token_type: TokenType::Colon,
                source_location: SourceLocation {
                    line: 1,
                    position: 33,
                },
            },
            Token {
                token_type: TokenType::F32,
                source_location: SourceLocation {
                    line: 1,
                    position: 35,
                },
            },
            Token {
                token_type: TokenType::RCurly,
                source_location: SourceLocation {
                    line: 1,
                    position: 39,
                },
            },
        ];

        assert_eq!(tokens.0, expected_tokens);
    }
}

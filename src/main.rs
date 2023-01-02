use fruko_bindgen::*;

fn main() {
    let tokens =
        lexer::lex_tokens(String::from("struct vector { x: f64, y64 }")).expect("should lex");
    let ast = parser::parse_tokens(tokens).expect("should parse");
}

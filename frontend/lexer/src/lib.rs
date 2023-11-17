pub use logos::{Logos, Lexer};

#[derive(Logos, Debug, Clone)]
pub enum Token {
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
    #[regex(r#""[^"]*""#, |lex| { let slice = lex.slice(); slice[1..slice.len()-1].to_owned() })]
    StringLiteral(String),
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token("@")]
    At
}

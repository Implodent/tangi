use super::*;
use std::{ops::Range, fmt::Debug};
use tangic_lexer::*;
use aott::input::SpannedInput;

#[derive(Debug, Clone, Copy, Default)]
pub struct Extra;

impl ParserExtras<TokenStream> for Extra {
    type Error = ParserError;
    type Context = ();
}

pub struct TokenStream {
    pub input: String,
    what: SpannedInput<Token, Range<usize>, Stream<Help>>,
}

impl Debug for TokenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "TokenStream {{ input: <redacted> }}")
    }
}

impl TokenStream {
    pub fn new(input: String) -> Result<Self, Vec<LexerError>> {
        let what = Stream::from_iter(Help::new(&input)?).spanned(input.len()-1..input.len());

        Ok(Self { input, what })
    }
}

impl InputType for TokenStream {
    type Token = Token;
    type OwnedMut = Help;
    type Offset = usize;
    type Span = Range<usize>;

    fn span(&self, span: Range<usize>) -> Self::Span {
        self.what.span(span)
    }
    fn prev(&self, offset: Self::Offset) -> Self::Offset {
        self.what.prev(offset)
    }
    fn start(&self) -> Self::Offset {
        self.what.start()
    }
    unsafe fn next(&self, offset: Self::Offset) -> (Self::Offset, Option<Self::Token>) {
        self.what.next(offset)
    }
}

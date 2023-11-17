use aott::{input::SpannedInput, prelude::*};

use std::ops::Range;

use tangic_lexer::*;

pub struct What<'a> {
    input: &'static str,
    what: SpannedInput<Token, Range<usize>, Stream<Lexer<'a, Token>>>
}

impl<'a> InputType for What<'a> {
    type Token = Token;
    type OwnedMut = Lexer<'a, Token>;
    type Offset = usize;
    type Span = Range<usize>;

    fn span(&self, span: Range<usize>) -> Self::Span {
        self.what.span(span)
    }
}

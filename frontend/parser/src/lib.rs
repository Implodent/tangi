use aott::{input::SpannedInput, prelude::*};

use std::ops::Range;

use tangic_lexer::*;

pub struct What {
    pub input: String,
    what: SpannedInput<Token, Range<usize>, Stream<Help>>,
}

impl InputType for What {
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

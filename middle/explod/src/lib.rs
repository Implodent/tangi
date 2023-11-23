//! explode
//! thx to zkat for original miette code, this is just an adapted version of it

mod fmter;
mod fmter_util;

#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Level {
    Bug,
    Fatal,
    Error,
    Warning,
    Note,
    OnceNote,
    Help,
    OnceHelp,
    FailNote,
    Allow
}

impl Default for Level {
    fn default() -> Self {
        Level::Error
    }
}

pub type ByteOffset = usize;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct SourceOffset(ByteOffset);

impl SourceOffset {
    /// Actual byte offset.
    pub const fn offset(&self) -> ByteOffset {
        self.0
    }

    /// Little utility to help convert 1-based line/column locations into
    /// miette-compatible Spans
    ///
    /// This function is infallible: Giving an out-of-range line/column pair
    /// will return the offset of the last byte in the source.
    pub fn from_location(source: impl AsRef<str>, loc_line: usize, loc_col: usize) -> Self {
        let mut line = 0usize;
        let mut col = 0usize;
        let mut offset = 0usize;
        for char in source.as_ref().chars() {
            if line + 1 >= loc_line && col + 1 >= loc_col {
                break;
            }
            if char == '\n' {
                col = 0;
                line += 1;
            } else {
                col += 1;
            }
            offset += char.len_utf8();
        }

        SourceOffset(offset)
    }
}

impl From<ByteOffset> for SourceOffset {
    fn from(bytes: ByteOffset) -> Self {
        SourceOffset(bytes)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Span {
    /// The start of the span.
    offset: SourceOffset,
    /// The total length of the span
    length: usize,
}

impl Span {
    /// Create a new [`SourceSpan`].
    pub const fn new(start: SourceOffset, length: SourceOffset) -> Self {
        Self {
            offset: start,
            length: length.offset(),
        }
    }

    /// The absolute offset, in bytes, from the beginning of a [`SourceCode`].
    pub const fn offset(&self) -> usize {
        self.offset.offset()
    }

    /// Total length of the [`SourceSpan`], in bytes.
    pub const fn len(&self) -> usize {
        self.length
    }

    /// Whether this [`SourceSpan`] has a length of zero. It may still be useful
    /// to point to a specific point.
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl From<(ByteOffset, usize)> for Span {
    fn from((start, len): (ByteOffset, usize)) -> Self {
        Self {
            offset: start.into(),
            length: len,
        }
    }
}

impl From<(SourceOffset, SourceOffset)> for Span {
    fn from((start, len): (SourceOffset, SourceOffset)) -> Self {
        Self::new(start, len)
    }
}

impl From<std::ops::Range<ByteOffset>> for Span {
    fn from(range: std::ops::Range<ByteOffset>) -> Self {
        Self {
            offset: range.start.into(),
            length: range.len(),
        }
    }
}

impl From<SourceOffset> for Span {
    fn from(offset: SourceOffset) -> Self {
        Self { offset, length: 0 }
    }
}

impl From<ByteOffset> for Span {
    fn from(offset: ByteOffset) -> Self {
        Self {
            offset: offset.into(),
            length: 0,
        }
    }
}

// ??????
pub type DiagnosticMessage = String;

pub struct MultiSpan {
    pub(crate) primaries: Vec<Span>,
    pub(crate) labels: Vec<(Span, DiagnosticMessage)>
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum DiagnosticId {
    Error(String),
    Lint {
        name: String,
        /// Indicates whether this lint should show up in v's future breakage report.
        has_future_breakage: bool,
        is_force_warn: bool,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
pub enum Style {
    MainHeaderMsg,
    HeaderMsg,
    LineAndColumn,
    LineNumber,
    Quotation,
    UnderlinePrimary,
    UnderlineSecondary,
    LabelPrimary,
    LabelSecondary,
    NoStyle,
    Level(Level),
    Highlight,
    Addition,
    Removal,
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Diagnostic {
    pub level: Level,

    pub message: Vec<(DiagnosticMessage, Style)>,
    pub code: Option<DiagnosticId>,
    pub span: MultiSpan,
}

/// A "sub"-diagnostic attached to a parent diagnostic.
/// For example, a note attached to an error.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct SubDiagnostic {
    pub level: Level,
    pub message: Vec<(DiagnosticMessage, Style)>,
    pub span: MultiSpan,
    pub render_span: Option<MultiSpan>,
}

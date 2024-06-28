//! Lexical tokens.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// Span of text.
///
/// Stores index and count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span(pub(crate) u32, pub(crate) u32);

impl Span {
    pub(crate) fn new(index: u32, count: u32) -> Self {
        Self(index, count)
    }

    pub fn fragment<'a>(&self, text: &'a str) -> &'a str {
        let Self(lo, hi) = *self;
        let lo = lo as usize;
        let hi = hi as usize;
        &text[lo..lo + hi]
    }

    pub fn index(&self) -> u32 {
        self.0
    }

    pub fn count(&self) -> u32 {
        self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum TokenKind {
    Comma,   // ,
    Dot,     // .
    Eq,      // =
    Hash,    // #
    Semi,    // ;

    Plus,    // +
    Minus,   // -
    Star,    // *
    Slash,   // /

    ParenLeft,    // (
    ParenRight,   // )
    BraceLeft,    // {
    BraceRight,   // }
    BracketLeft,  // [
    BracketRight, // ]

    Less,        // <
    LessEq,      // <=
    Great,       // >
    GreatEq,     // >=

    Ident,   // identifier
    Num,     // integer literal
    Str,     // string literal
    Doc,     // document comment

    Keyword(Keyword),

    Eof,     // End-of-file
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Fn,
    For,
    Let,
    If,
    Import,
    Type,
    While,
}

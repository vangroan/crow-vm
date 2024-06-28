//! Lexical tokens.
use std::fmt::{self, Formatter};

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub lit: Option<LitValue>,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span, lit: None }
    }

    pub const fn new_lit(kind: TokenKind, span: Span, lit: LitValue) -> Self {
        Self {
            kind,
            span,
            lit: Some(lit),
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        // Invariant: Literal value omitted because float equality
        //            is problematic and the span should be enough
        //            to identify a token.
        self.kind == other.kind && self.span == other.span
    }
}

impl Eq for Token {}

#[derive(Debug, Clone, PartialEq)]
pub enum LitValue {
    Int(i64),
    Float(f64),
    Str(String),
}

impl fmt::Display for LitValue {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            LitValue::Int(value) => fmt::Display::fmt(value, f),
            LitValue::Float(value) => fmt::Display::fmt(value, f),
            LitValue::Str(value) => fmt::Display::fmt(value, f),
        }
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

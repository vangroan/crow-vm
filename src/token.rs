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
    Comma,    // ,
    Dot,      // .
    Eq,       // =
    EqEq,     // ==
    NotEq,    // !=
    Hash,     // #
    Colon,    // :
    Semi,     // ;
    Perc,     // %

    Plus,     // +
    Minus,    // -
    Star,     // *
    StarStar, // **
    Slash,    // /

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

    Kw(Keyword),

    Eof,     // End-of-file
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    And,
    Fn,
    For,
    Let,
    If,
    Import,
    Or,
    Struct,
    Type,
    While,
}

/// Token operator precedence.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
pub enum Precedence {
    /// Tokens that terminate an expression
    /// should have a precedence of `None`.
    None = 0,
    Lowest = 1,
    Assignment = 2,    // =
    Conditional = 3,   // ?:
    LogicalOr = 4,     // || or
    LogicalAnd = 5,    // && and
    Equality = 6,      // == !=
    Is = 7,            // is
    Comparison = 8,    // < > <= >=
    BitwiseOr = 9,     // |
    BitwiseXor = 10,   // ^
    BitwiseAnd = 11,   // &
    BitwiseShift = 12, // << >>
    Range = 13,        // .. ...
    Term = 14,         // + -
    Factor = 15,       // * / %
    Unary = 16,        // - ! ~
    Exponent = 17,     // **
    Call = 18,         // . () []
    Primary = 19,
}

impl Precedence {
    #[inline(always)]
    fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// Get the precedence of the given token type in the context
    /// of the expression parser.
    pub fn of(kind: TokenKind) -> Precedence {
        use self::TokenKind::*;

        match kind {
            Num | Ident => Precedence::Lowest,
            Plus | Minus => Precedence::Term,
            Star | Slash => Precedence::Factor,
            StarStar => Precedence::Exponent,
            Eq => Precedence::Assignment,
            EqEq => Precedence::Equality,
            Dot | ParenLeft | BracketLeft => Precedence::Call,
            // ------------------------------------------------
            // Terminators
            ParenRight | BracketRight => Precedence::None,
            Semi => Precedence::None,
            Comma => Precedence::None,
            Eof => Precedence::None,
            _ => Precedence::None,
        }
    }
}

impl From<i32> for Precedence {
    #[rustfmt::skip]
    fn from(value: i32) -> Self {
        use Precedence as P;
        match value {
            0  => P::None,
            1  => P::Lowest,
            2  => P::Assignment,
            3  => P::Conditional,
            4  => P::LogicalOr,
            5  => P::LogicalAnd,
            6  => P::Equality,
            7  => P::Is,
            8  => P::Comparison,
            9  => P::BitwiseOr,
            10 => P::BitwiseXor,
            11 => P::BitwiseAnd,
            12 => P::BitwiseShift,
            13 => P::Range,
            14 => P::Term,
            15 => P::Factor,
            16 => P::Unary,
            17 => P::Exponent,
            18 => P::Call,
            19 => P::Primary,
            _  => P::None,
        }
    }
}

impl std::fmt::Display for Precedence {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.as_i32(), f)
    }
}

impl std::ops::Add<i32> for Precedence {
    type Output = Precedence;

    fn add(self, rhs: i32) -> Self::Output {
        Precedence::from(self.as_i32() + rhs)
    }
}

/// Associativity is the precedence tie-breaker.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Associativity {
    Left,
    Right,
}

impl Associativity {
    /// Determine the associativity of the given token kind.
    pub fn of(token_ty: TokenKind) -> Associativity {
        // Assignment and exponent are right associative.
        if matches!(token_ty, TokenKind::Eq | TokenKind::StarStar) {
            Associativity::Right
        } else {
            Associativity::Left
        }
    }

    pub fn is_left(&self) -> bool {
        *self == Associativity::Left
    }
}

//! Lexical analyser.
use crate::errors::{lexer_err, Result};
use crate::token::{Keyword, LitValue, Span, Token, TokenKind};

macro_rules! trace {
    ($($arg:tt)*) => {
        if cfg!(feature = "trace_lexer") {
            println!($($arg)*);
        }
    };
}

/// Lexical analyser.
pub struct Lexer<'a> {
    /// Original source code text.
    text: &'a str,
    /// Remaining source code text to be lexed.
    rest: &'a str,
    /// Span of the text fragment that was consumed. `(byte_offset, size)`
    span: Span,
    /// File where the source text is from.
    pub(crate) file: Option<String>,
}

impl<'a> Lexer<'a> {
    /// Create a new [`Lexer`] from the given source code.
    ///
    /// The filename (or any debug name) can be provided to
    /// provide clearer error messages.
    pub fn new(text: &'a str, file: impl ToString) -> Self {
        Self {
            text,
            rest: text,
            span: Span::new(0, 0),
            file: Some(file.to_string()),
        }
    }

    /// Creates a new [`Lexer`] from the given source code
    /// with no debug file name.
    #[allow(dead_code)]
    pub(crate) fn from_source(text: &'a str) -> Self {
        Self {
            text,
            rest: text,
            span: Span::new(0, 0),
            file: None,
        }
    }

    pub fn text(&self) -> &str {
        self.text
    }

    pub fn next_token(&mut self) -> Result<Token> {
        use crate::token::TokenKind::*;

        loop {
            self.ignore_whitespace();
            self.start_token();

            let token = match self.bump() {
                Some((_, ch)) => match ch {
                    '0'..='9' => self.lex_number()?,
                    'a'..='z' | 'A'..='Z' => self.lex_ident(),

                    // --------------------------------------------------------
                    // Punctuation
                    ',' => self.make_token(Comma),
                    '.' => self.make_token(Dot),
                    '=' => {
                        if self.match_char('=') {
                            self.make_token(EqEq)
                        } else {
                            self.make_token(Eq)
                        }
                    }
                    '!' => {
                        if self.match_char('=') {
                            self.make_token(NotEq)
                        } else {
                            return lexer_err(format!("unexpected character {ch:?}")).into();
                        }
                    }
                    '#' => self.make_token(Hash),
                    ':' => self.make_token(Colon),
                    ';' => self.make_token(Semi),
                    '%' => self.make_token(Perc),

                    // --------------------------------------------------------
                    // Operators
                    '+' => self.make_token(Plus),
                    '-' => self.make_token(Minus),
                    '*' => {
                        if self.match_char('*') {
                            self.make_token(StarStar)
                        } else {
                            self.make_token(Star)
                        }
                    }
                    '/' => {
                        // Comments
                        if self.match_char('/') {
                            if self.match_char('/') {
                                self.lex_doc_comment()
                            } else {
                                self.ignore_line_comment();
                                continue;
                            }
                        } else if self.match_char('*') {
                            self.ignore_block_comment();
                            continue;
                        } else {
                            self.make_token(Slash)
                        }
                    }

                    // --------------------------------------------------------
                    // Enclosures
                    '(' => self.make_token(ParenLeft),
                    ')' => self.make_token(ParenRight),
                    '{' => self.make_token(BraceLeft),
                    '}' => self.make_token(BraceRight),
                    '[' => self.make_token(BracketLeft),
                    ']' => self.make_token(BracketRight),
                    '"' => self.lex_string_literal(),

                    // --------------------------------------------------------
                    // Comparison
                    '<' => {
                        if self.match_char('=') {
                            self.make_token(LessEq)
                        } else {
                            self.make_token(Less)
                        }
                    }
                    '>' => {
                        if self.match_char('=') {
                            self.make_token(GreatEq)
                        } else {
                            self.make_token(Great)
                        }
                    }

                    _ => return lexer_err(format!("unexpected character {ch:?}")).into(),
                },
                // End-of-file
                None => self.make_token(TokenKind::Eof),
            };

            return Ok(token);
        }
    }

    /// Strign fragment of the current span.
    fn fragment(&self) -> &str {
        let lo = self.span.0 as usize;
        let hi = self.span.1 as usize;
        &self.text[lo..(lo + hi)]
    }

    /// Bump the cursor to the next character.
    fn bump(&mut self) -> Option<(usize, char)> {
        match self.rest.chars().next() {
            Some(c) => {
                // Length in bytes when UTF-8 encoded.
                let char_len = c.len_utf8();
                self.rest = &self.rest[char_len..];
                self.span.1 += char_len as u32;
                Some((self.pos(), c))
            }
            None => None,
        }
    }

    fn match_char(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.bump();
            true
        } else {
            false
        }
    }

    /// Peek the next character without advancing the cursor.
    fn peek(&self) -> Option<char> {
        self.rest.chars().next()
    }

    /// Peek the character after next without advancing the cursor.
    fn peek2(&self) -> Option<char> {
        let mut chars = self.rest.chars();
        chars.next();
        chars.next()
    }

    /// Current position in the source text.
    fn pos(&self) -> usize {
        (self.rest.as_ptr() as usize) - (self.text.as_ptr() as usize)
    }

    /// Setup the lexer to create a new token.
    fn start_token(&mut self) {
        self.span = Span(self.pos() as u32, 0);
        trace!("start token at {}:", self.span.0);
    }

    /// Finishes the current token.
    ///
    /// See [`Lexer::start_token()`].
    fn make_token(&mut self, kind: TokenKind) -> Token {
        trace!(
            "    {}:{} {kind:?} {:?}",
            self.span.0,
            self.span.0 + self.span.1,
            self.fragment(),
        );
        Token::new(kind, self.span.clone())
    }

    fn make_literal(&mut self, kind: TokenKind, literal_value: LitValue) -> Token {
        trace!(
            "    {}:{} {kind:?} {}",
            self.span.0,
            self.span.0 + self.span.1,
            self.fragment(),
        );
        Token::new_lit(kind, self.span.clone(), literal_value)
    }
}

impl<'a> Lexer<'a> {
    /// Ignore all whitespace. Newlines are not significant to this language.
    fn ignore_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn ignore_line_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch != '\n' {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn ignore_block_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '*' {
                if self.peek2() == Some('/') {
                    self.bump();
                    self.bump();
                    break;
                }
            }
            self.bump();
        }
    }

    fn lex_doc_comment(&mut self) -> Token {
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == '\n' {
                break;
            }
        }

        self.make_token(TokenKind::Doc)
    }

    #[rustfmt::skip]
    fn try_keyword(&self) -> Option<Keyword> {
        use crate::token::Keyword::*;

        match self.fragment() {
            "and"    => Some(And),
            "fn"     => Some(Fn),
            "for"    => Some(For),
            "let"    => Some(Let),
            "if"     => Some(If),
            "import" => Some(Import),
            "or"     => Some(Or),
            "struct" => Some(Struct),
            "type"   => Some(Type),
            "while"  => Some(While),
            _ => None,
        }
    }

    /// Numbers are sequences of digits.
    fn lex_number(&mut self) -> Result<Token> {
        // trace!("    lex_number()");

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }

        let fragment = self.fragment();
        let value = i64::from_str_radix(fragment, 10)
            .map(LitValue::Int)
            .map_err(|err| lexer_err(format!("failed to parser number literal: {err}")))?;

        Ok(self.make_literal(TokenKind::Num, value))
    }

    /// Identifiers start with a letter or underscore,
    /// then can contain letters, digits and underscores.
    fn lex_ident(&mut self) -> Token {
        // trace!("    lex_ident()");

        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.bump();
            } else {
                break;
            }
        }

        let kind = match self.try_keyword() {
            Some(keyword) => TokenKind::Kw(keyword),
            None => TokenKind::Ident,
        };

        self.make_token(kind)
    }

    fn lex_string_literal(&mut self) -> Token {
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            self.bump();
            if ch == '"' {
                break;
            } else {
                value.push(ch);
            }
        }

        self.make_literal(TokenKind::Str, LitValue::Str(value))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::Result;
    use crate::token::Keyword::*;
    use crate::token::TokenKind::*;

    /// Shorthand convenience function for creating a token.
    fn token(kind: TokenKind, span: (u32, u32)) -> Token {
        Token::new(kind, Span(span.0, span.1))
    }

    /// Shorthand convenience function for creating a keyword token.
    fn keyword(kind: crate::token::Keyword, span: (u32, u32)) -> Token {
        Token::new(TokenKind::Kw(kind), Span(span.0, span.1))
    }

    #[test]
    #[rustfmt::skip]
    fn test_tokenisation_punctuation() -> Result<()> {
        let mut lexer = Lexer::from_source(", . = # ;");

        assert_eq!(lexer.next_token()?, token(Comma, (0, 1)));
        assert_eq!(lexer.next_token()?, token(Dot,   (2, 1)));
        assert_eq!(lexer.next_token()?, token(Eq,    (4, 1)));
        assert_eq!(lexer.next_token()?, token(Hash,  (6, 1)));
        assert_eq!(lexer.next_token()?, token(Semi,  (8, 1)));

        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn test_tokenisation_operators() -> Result<()> {
        let mut lexer = Lexer::from_source("+ - * /");

        assert_eq!(lexer.next_token()?, token(Plus,  (0, 1)));
        assert_eq!(lexer.next_token()?, token(Minus, (2, 1)));
        assert_eq!(lexer.next_token()?, token(Star,  (4, 1)));
        assert_eq!(lexer.next_token()?, token(Slash, (6, 1)));

        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn test_tokenisation_enclosing() -> Result<()> {
        let mut lexer = Lexer::from_source("( ) { } [ ]");

        assert_eq!(lexer.next_token()?, token(ParenLeft,    (0, 1)));
        assert_eq!(lexer.next_token()?, token(ParenRight,   (2, 1)));
        assert_eq!(lexer.next_token()?, token(BraceLeft,    (4, 1)));
        assert_eq!(lexer.next_token()?, token(BraceRight,   (6, 1)));
        assert_eq!(lexer.next_token()?, token(BracketLeft,  (8, 1)));
        assert_eq!(lexer.next_token()?, token(BracketRight, (10, 1)));

        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn test_tokenisation_comparison() -> Result<()> {
        let mut lexer = Lexer::from_source("< <= > >=");

        assert_eq!(lexer.next_token()?, token(Less,    (0, 1)));
        assert_eq!(lexer.next_token()?, token(LessEq,  (2, 2)));
        assert_eq!(lexer.next_token()?, token(Great,   (5, 1)));
        assert_eq!(lexer.next_token()?, token(GreatEq, (7, 2)));

        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn test_tokenisation_keywords() -> Result<()> {
        let mut lexer = Lexer::from_source("and fn for let if import or struct type while");

        assert_eq!(lexer.next_token()?, keyword(And,    (0, 3)));
        assert_eq!(lexer.next_token()?, keyword(Fn,     (4, 2)));
        assert_eq!(lexer.next_token()?, keyword(For,    (7, 3)));
        assert_eq!(lexer.next_token()?, keyword(Let,    (11, 3)));
        assert_eq!(lexer.next_token()?, keyword(If,     (15, 2)));
        assert_eq!(lexer.next_token()?, keyword(Import, (18, 6)));
        assert_eq!(lexer.next_token()?, keyword(Or,     (25, 2)));
        assert_eq!(lexer.next_token()?, keyword(Struct, (28, 6)));
        assert_eq!(lexer.next_token()?, keyword(Type,   (35, 4)));
        assert_eq!(lexer.next_token()?, keyword(While,  (40, 5)));

        Ok(())
    }

    #[test]
    fn test_ignore_line_comment() -> Result<()> {
        let mut lexer = Lexer::from_source("a \n //foobar \n b");

        assert_eq!(lexer.next_token()?, token(Ident, (0, 1)));
        assert_eq!(lexer.next_token()?, token(Ident, (15, 1)));

        Ok(())
    }

    #[test]
    fn test_ignore_block_comment() -> Result<()> {
        let mut lexer = Lexer::from_source("a \n /* foobar */ \n b");

        assert_eq!(lexer.next_token()?, token(Ident, (0, 1)));
        assert_eq!(lexer.next_token()?, token(Ident, (19, 1)));

        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn test_doc_comment() -> Result<()> {
        let mut lexer = Lexer::from_source("a \n /// foobar \n b");

        assert_eq!(lexer.next_token()?, token(Ident, (0, 1)));
        assert_eq!(lexer.next_token()?, token(Doc,   (4, 12)));
        assert_eq!(lexer.next_token()?, token(Ident, (17, 1)));

        Ok(())
    }
}

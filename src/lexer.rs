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
                    '0'..='9' => self.lex_number(),
                    'a'..='z' | 'A'..='Z' => self.lex_ident(),

                    ',' => self.make_token(Comma),
                    '.' => self.make_token(Dot),
                    '=' => self.make_token(Eq),
                    '#' => self.make_token(Hash),
                    ';' => self.make_token(Semi),

                    '+' => self.make_token(Plus),
                    '-' => self.make_token(Minus),
                    '*' => self.make_token(Star),
                    '/' => {
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

                    '(' => self.make_token(ParenLeft),
                    ')' => self.make_token(ParenRight),
                    '{' => self.make_token(BraceLeft),
                    '}' => self.make_token(BraceRight),
                    '[' => self.make_token(BracketLeft),
                    ']' => self.make_token(BracketRight),

                    '"' => self.lex_string_literal(),

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
            "fn"     => Some(Fn),
            "for"    => Some(For),
            "let"    => Some(Let),
            "if"     => Some(If),
            "import" => Some(Import),
            "type"   => Some(Type),
            "while"  => Some(While),
            _ => None,
        }
    }

    /// Numbers are sequences of digits.
    fn lex_number(&mut self) -> Token {
        // trace!("    lex_number()");

        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.bump();
            } else {
                break;
            }
        }

        self.make_token(TokenKind::Num)
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
            Some(keyword) => TokenKind::Keyword(keyword),
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
    use crate::token::TokenKind::*;

    /// Shorthand convenience function for creating a token.
    fn token(kind: TokenKind, span: (u32, u32)) -> Token {
        Token::new(kind, Span(span.0, span.1))
    }

    #[test]
    fn test_tokenisation() {
        let mut lexer = Lexer::from_source(", . = # ;");

        assert_eq!(lexer.next_token().unwrap(), token(Comma, (0, 1)));
        assert_eq!(lexer.next_token().unwrap(), token(Dot, (2, 1)));
        assert_eq!(lexer.next_token().unwrap(), token(Eq, (4, 1)));
        assert_eq!(lexer.next_token().unwrap(), token(Hash, (6, 1)));
        assert_eq!(lexer.next_token().unwrap(), token(Semi, (8, 1)));
    }

    #[test]
    fn test_ignore_line_comment() {
        let mut lexer = Lexer::from_source("a \n //foobar \n b");

        assert_eq!(lexer.next_token().unwrap(), token(Ident, (0, 1)));
        assert_eq!(lexer.next_token().unwrap(), token(Ident, (15, 1)));
    }

    #[test]
    fn test_ignore_block_comment() {
        let mut lexer = Lexer::from_source("a \n /* foobar */ \n b");

        assert_eq!(lexer.next_token().unwrap(), token(Ident, (0, 1)));
        assert_eq!(lexer.next_token().unwrap(), token(Ident, (19, 1)));
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

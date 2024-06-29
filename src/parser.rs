//! Syntactic parser.
use crate::ast::*;
use crate::errors::{parser_err, Result};
use crate::lexer::Lexer;
use crate::token::{Token, TokenKind};
use crate::types::TypeId;

/// Syntactic parser.
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    /// The current token, if the next has been peeked.
    token: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self { lexer, token: None }
    }

    fn next_token(&mut self) -> Result<Token> {
        match self.token.take() {
            Some(token) => Ok(token),
            None => self.lexer.next_token(),
        }
    }

    fn peek_token(&mut self) -> Result<&Token> {
        if self.token.is_some() {
            return self.token.as_ref().map(Ok).unwrap();
        }

        self.token = Some(self.lexer.next_token()?);
        self.token.as_ref().map(Ok).unwrap()
    }

    fn peek_kind(&mut self) -> Result<TokenKind> {
        self.peek_token().map(|token| token.kind)
    }

    /// Consume the next token that matches the given token kind.
    ///
    /// Returns an error if the token does not match.
    fn consume_token(&mut self, token_kind: TokenKind) -> Result<Token> {
        let actual_kind = self.peek_kind()?;
        if actual_kind == token_kind {
            self.next_token()
        } else {
            parser_err(format!("expected token {:?}, found {:?}", token_kind, actual_kind)).into()
        }
    }

    fn match_token(&mut self, token_kind: TokenKind) -> Result<bool> {
        if self.peek_kind()? == token_kind {
            self.next_token()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Parse the source text as if its a top-level module file.
    pub fn parse_module(&mut self) -> Result<Block> {
        // A module is syntactically identical to a block body.
        Ok(Block {
            ty: TypeId::default(),
            stmts: self.parse_stmts()?,
        })
    }

    /// Parse zero or more statements.
    fn parse_stmts(&mut self) -> Result<Vec<Stmt>> {
        use crate::token::{Keyword::*, TokenKind::*};

        let mut stmts = Vec::new();

        loop {
            let token = self.next_token()?;

            let stmt = match token.kind {
                Keyword(Let) => self.parse_let_stmt().map(Box::new).map(Stmt::Local)?,
                Ident => self.parse_expr_stmt(token).map(Box::new).map(Stmt::Expr)?,
                Eof => break,
                _ => return parser_err(format!("unexpected token: {:?}", token.kind)).into(),
            };

            stmts.push(stmt);
        }

        Ok(stmts)
    }

    /// Parse a local variable declaration statement.
    fn parse_let_stmt(&mut self) -> Result<LocalDecl> {
        let name = self.parse_ident()?;

        let ty = if self.match_token(TokenKind::Colon)? {
            self.parse_type_def().map(Some)?
        } else {
            None
        };

        let rhs = if self.match_token(TokenKind::Eq)? {
            self.parse_expr().map(Some)?
        } else {
            None
        };

        Ok(LocalDecl { name, ty, rhs })
    }

    /// Parse an expression statement.
    ///
    /// Only a subset of expression may be valid statements.
    fn parse_expr_stmt(&mut self, _token: Token) -> Result<Expr> {
        todo!("expression statement")
    }
}

impl<'a> Parser<'a> {
    fn parse_type_def(&mut self) -> Result<TypeDef> {
        todo!("parse type definition")
    }
}

impl<'a> Parser<'a> {
    /// Parse an expression.
    pub fn parse_expr(&mut self) -> Result<Expr> {
        todo!("parse expression")
    }

    fn parse_ident(&mut self) -> Result<Ident> {
        let token = self.consume_token(TokenKind::Ident)?;
        let fragment = token.span.fragment(self.lexer.text());
        Ok(Ident {
            text: fragment.to_string(),
        })
    }

    fn parse_func_lit(&mut self) -> Result<FuncLit> {
        todo!("parse function literal")
    }
}

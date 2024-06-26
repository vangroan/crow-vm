//! Syntactic parser.
use crate::ast::*;
use crate::errors::{parser_err, Result};
use crate::lexer::Lexer;
use crate::token::{Associativity, LitValue, Precedence, Token, TokenKind};
use crate::types::TypeId;

macro_rules! trace {
    ($($arg:tt)*) => {
        if cfg!(feature = "trace_parser") {
            println!($($arg)*);
        }
    };
}

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
                Kw(Let) => self.parse_let_stmt().map(Box::new).map(Stmt::Local)?,
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

        self.consume_token(TokenKind::Semi)?;

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
        trace!("parse_expr");
        self.parse_precedence(Precedence::Lowest)
    }

    /// Entrypoint for the top-down precedence parser.
    ///
    /// The implementation is a straight forward Pratt parser.
    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr> {
        trace!("parse_precedence({precedence:?})");

        let token = self.next_token()?;
        trace!("parse_precedence(..); token -> {token:?}");

        let mut left = self.parse_prefix(token)?;

        while precedence <= self.peek_kind().map(|kind| Precedence::of(kind))? {
            // When thre is no expression right of the last one, we just return what we have.
            let op = self.next_token()?;
            left = self.parse_infix(left, op).map(Box::new).map(Expr::Binary)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self, token: Token) -> Result<Expr> {
        trace!("parse_prefix({token:?})");

        use crate::token::{Keyword::*, TokenKind::*};

        match token.kind {
            Num => self.parse_num_lit(token).map(Literal::Num).map(Box::new).map(Expr::Lit),
            Ident => self.parse_postfix(token),
            BracketLeft => todo!("array literal"),
            BraceLeft => todo!("table literal"),
            Kw(Fn) => self.parse_func_lit().map(Box::new).map(Expr::Func),
            _ => parser_err("expression expected").into(),
        }
    }

    fn parse_infix(&mut self, left: Expr, op: Token) -> Result<BinaryExpr> {
        use crate::token::TokenKind::*;
        trace!("parse_infix({left:?}, {op:?})");

        let precedence = Precedence::of(op.kind);

        // Associativity is handled by adjusting the precedence.
        // Left associativity is achieved by increasing the precedence
        // by 1. This increases the threshold that any infix expressions
        // to our right must exceed.
        //
        // Right associativity can be achieved by keeping
        // the precedence the same, thus keeping the threshold any
        // subsequent infix expression need to exceed to be parsed.
        let binding_power = if Associativity::of(op.kind).is_left() { 1 } else { 0 };

        // Recurse back into expression parser to handle
        // the right hand side.
        //
        // The left hand side will wait for us here on
        // the call stack.
        let right = self.parse_precedence(precedence + binding_power)?;

        match op.kind {
            // Binary Operations
            Plus | Minus | Star | Slash | StarStar | Eq | EqEq | NotEq => Ok(BinaryExpr {
                op: Self::parse_binary_op(op.kind)?,
                lhs: left,
                rhs: right,
            }),
            _ => parser_err("infix operator expected").into(),
        }
    }

    /// Parse a postfix expression.
    fn parse_postfix(&mut self, token: Token) -> Result<Expr> {
        trace!("parse_postfix({token:?})");
        assert_eq!(
            token.kind,
            TokenKind::Ident,
            "a postfix expression must start with an identifier"
        );

        // The parselet is triggered by encountering an identifier
        // in an expression as a prefix.
        //
        // The simplest case is the expression is referencing a variable.
        let mut _expr = Expr::Name(Box::new(NameAccessExpr {
            ident: self.make_ident(&token),
        }));

        // Keep parsing the rest of the expression to transform `expr`
        // into something else.
        loop {
            match self.peek_kind()? {
                TokenKind::Eq => todo!("assignment"),
                TokenKind::BracketLeft => todo!("subscript"),
                TokenKind::ParenLeft => todo!("call"),
                TokenKind::Dot => todo!("member access"),
                _ => break,
            }
        }

        todo!("postfix expression")
    }

    fn parse_binary_op(op_kind: TokenKind) -> Result<BinaryOp> {
        match op_kind {
            TokenKind::Plus => Ok(BinaryOp::Add),
            TokenKind::Minus => Ok(BinaryOp::Sub),
            TokenKind::Star => Ok(BinaryOp::Mul),
            TokenKind::Slash => Ok(BinaryOp::Div),
            TokenKind::Perc => Ok(BinaryOp::Mod),
            TokenKind::StarStar => Ok(BinaryOp::Exp),
            TokenKind::Eq => Ok(BinaryOp::Assign),
            _ => parser_err("invalid token for binary operation").into(),
        }
    }

    fn parse_num_lit(&mut self, token: Token) -> Result<Number> {
        match token.lit {
            Some(LitValue::Int(value)) => Ok(Number::Int(value)),
            Some(LitValue::Float(value)) => Ok(Number::Float(value)),
            Some(_) => parser_err("expected number literal value in token, found string literal value").into(),
            None => parser_err("expected number literal value in token, found none").into(),
        }
    }

    fn parse_ident(&mut self) -> Result<Ident> {
        let token = self.consume_token(TokenKind::Ident)?;
        Ok(self.make_ident(&token))
    }

    fn make_ident(&self, token: &Token) -> Ident {
        let fragment = token.span.fragment(self.lexer.text());
        Ident {
            text: fragment.to_string(),
        }
    }

    fn parse_func_lit(&mut self) -> Result<FuncLit> {
        todo!("parse function literal")
    }
}

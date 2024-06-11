use crate::types::TypeId;

/// Block of statements between two curly braces.
///
/// A block has a return type.
///
/// ```text
/// {
///   let x = b + y;
///   x
/// }
/// ```
#[derive(Debug)]
pub struct Block {
    /// Return value type.
    pub ty: TypeId,
    /// Statements.
    pub stmts: Vec<Stmt>,
}

// ============================================================================ //
// Statements                                                                   //
// ============================================================================ //

#[derive(Debug)]
pub enum Stmt {
    /// Local variable declaration.
    Local(Box<LocalDecl>),
    /// Explicit or implicit return statement.
    Return,
    /// Expression statement.
    Expr(Box<Expr>),
}

#[derive(Debug)]
pub struct LocalDecl {
    pub name: Ident,
    pub rhs: Option<Expr>,
}

#[derive(Debug)]
pub struct ReturnStmt {
    /// Return value type.
    ///
    /// [`crate::types::Type::Void`] when nothing is returned.
    pub ty: TypeId,
    /// Zero or more values to return.
    pub value: Tuple,
}

/// List of multiple values to return from a block or function.
#[derive(Debug)]
pub struct Tuple {
    pub items: Vec<TupleItem>,
}

#[derive(Debug)]
pub struct TupleItem {
    pub ty: TypeId,
    pub expr: Expr,
}

// ============================================================================ //
// Expressions                                                                  //
// ============================================================================ //

/// Expressions.
#[derive(Debug)]
pub enum Expr {
    Func(Box<FuncExpr>),
    /// Call to a closure.
    Call(Box<CallExpr>),
}

#[derive(Debug)]
pub struct FuncExpr {
    pub ty: TypeId,
    pub args: Vec<Arg>,
    pub return_: Tuple,
}

#[derive(Debug)]
pub struct Arg {
    pub name: Ident,
    pub ty_name: Ident,
}

/// Call expression/
#[derive(Debug)]
pub struct CallExpr {
    pub ty: TypeId,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
}

#[derive(Debug)]
pub struct Ident {
    pub text: String,
}

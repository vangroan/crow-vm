use crate::types::{TypeId, TYPE_FLOAT_ID, TYPE_INT_ID, TYPE_STRING_ID};

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
    pub ty: Option<TypeDef>,
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
    Binary(Box<BinaryExpr>),
    Lit(Box<Literal>),
    Func(Box<FuncLit>),
    /// Call to a closure.
    Call(Box<CallExpr>),
}

#[derive(Debug)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub lhs: Expr,
    pub rhs: Expr,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Exp,
    Assign,
}

/// Function definition literal.
#[derive(Debug)]
pub struct FuncLit {
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

// ============================================================================ //
// Common                                                                       //
// ============================================================================ //

#[derive(Debug)]
pub struct Ident {
    pub text: String,
}

#[derive(Debug)]
pub enum Literal {
    Num(Number),
    Str(String),
}

#[derive(Debug)]
pub enum Number {
    Int(i64),
    Float(f64),
}

// ============================================================================ //
// Types                                                                        //
// ============================================================================ //

/// Type declaration statement.
///
/// ```text
/// type <name> = <type-def>;
/// ```
#[derive(Debug)]
pub struct TypeDeclStmt {
    pub name: Ident,
    pub rhs: TypeDef,
}

/// Type definition.
///
/// ```text
/// <alias|literal>
/// ```
#[derive(Debug)]
pub enum TypeDef {
    Alias(TypeName),
    Lit(TypeLit),
}

/// A simple type name (an alias) to another existing type.
#[derive(Debug)]
pub struct TypeName {
    pub text: Ident,
}

/// Type literal.
#[derive(Debug)]
pub enum TypeLit {
    /// Array type literal.
    ///
    /// ```text
    /// [<type-def>; <number-lit>]
    /// ```
    Array { element: Box<TypeDef>, size: usize },

    /// Dynamic array type literal.
    ///
    /// ```text
    /// "[" <type-def> "]""
    /// ```
    DynArray { element: Box<TypeDef> },

    /// Hash table type literal.
    ///
    /// ```text
    /// "{" <type-def> ":" <type-def> "}"
    /// ```
    Table { key: Box<TypeDef>, value: Box<TypeDef> },

    /// Structure type literal.
    ///
    /// ```text
    /// "struct" "{" <field-def> ("," <field-def>)* "}"
    /// ```
    Struct { fields: Vec<FieldDef> },
}

/// Struct type field definition.
#[derive(Debug)]
pub struct FieldDef {
    pub name: Ident,
    pub ty: Box<TypeDef>,
}

// ============================================================================ //
// Functions                                                                    //
// ============================================================================ //

impl Ident {
    pub fn from_string(text: impl ToString) -> Self {
        Ident { text: text.to_string() }
    }
}

impl Literal {
    pub fn type_id(&self) -> TypeId {
        match self {
            Literal::Num(Number::Int(_)) => TYPE_INT_ID,
            Literal::Num(Number::Float(_)) => TYPE_FLOAT_ID,
            Literal::Str(_) => TYPE_STRING_ID,
        }
    }
}

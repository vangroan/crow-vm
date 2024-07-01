use std::collections::HashMap;

use crate::ast::*;
use crate::errors::{typecheck_err, Result};
use crate::types::*;

pub struct TypeChecker {
    types: Vec<Type>,
    aliases: HashMap<String, TypeId>,
    scope: Scope,
    scopes: Vec<Scope>,
}

struct Scope {
    /// Local variables declared in this scope.
    locals: Vec<Local>,
}

struct Local {
    name: String,
    ty: TypeId,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self {
            types: init_type_table(),
            aliases: init_type_aliases(),
            scope: Scope { locals: vec![] },
            scopes: vec![],
        }
    }

    /// Resolve a type from a type definition syntax node.
    ///
    /// If the type definition is a literal, an existing definition
    /// will be looked up in the type table.
    ///
    /// If an existing definition isn't found, and new type is
    /// defined with a new [`TypeId`].
    fn resolve_type(&mut self, type_def: &TypeDef) -> Result<TypeId> {
        use crate::ast::TypeLit::*;

        match type_def {
            // The simple case is to lookup the type alias by string.
            TypeDef::Alias(name) => self
                .aliases
                .get(name.text.text.as_str())
                .cloned()
                .ok_or_else(|| typecheck_err(format!("unknown type alias: {}", name.text.text))),
            TypeDef::Lit(Array { .. }) => todo!(),
            TypeDef::Lit(DynArray { .. }) => todo!(),
            TypeDef::Lit(Table { .. }) => todo!(),
            TypeDef::Lit(Struct { .. }) => todo!(),
        }
    }

    /// Type check the given block.
    pub fn check_block(&mut self, block: &Block) -> Result<TypeId> {
        // TODO: Collect all the return types to determin the block's return type.
        for stmt in &block.stmts {
            // The resulting type of a statement is discarded.
            self.check_stmt(stmt)?;
        }

        // Block with no return will return void.
        Ok(TYPE_VOID_ID)
    }

    /// Type check all the given statements.
    pub fn check_stmt(&mut self, stmt: &Stmt) -> Result<TypeId> {
        match stmt {
            Stmt::Local(local_decl) => self.check_local_decl(local_decl),
            Stmt::Return => todo!(),
            Stmt::Expr(_) => todo!(),
        }
    }

    /// Type check the given local variable declaration.
    ///
    /// Variable declaration has three forms:
    ///
    /// 1. No type, RHS expression
    /// 2. Type, no RHS expression
    /// 3. Type and RHS expression
    ///
    /// A local variable declaration with no type and no right hand side expression is invalid.
    fn check_local_decl(&mut self, local_decl: &LocalDecl) -> Result<TypeId> {
        // Type is explicitly user defined.
        let maybe_ty = match &local_decl.ty {
            Some(type_lit) => Some(self.resolve_type(type_lit)?),
            None => None,
        };

        // Initial value is defined with an expression.
        let maybe_rhs_ty = match &local_decl.rhs {
            Some(expr) => Some(self.check_expr(&expr)?),
            None => None,
        };

        match (maybe_ty, maybe_rhs_ty) {
            // No type nor expression defined.
            (None, None) => typecheck_err(format!(
                "local variable declaration needs an explicit type, or an initial value"
            ))
            .into(),
            // Void cannot be used as a value.
            (_, Some(TYPE_VOID_ID)) => typecheck_err(format!("Void cannot be assigned to a variable")).into(),
            // Type inference.
            (None, Some(ty)) => {
                self.declare_local(local_decl.name.text.clone(), ty);
                Ok(ty)
            }
            (Some(ty), None) => {
                // TODO: No init value. RHS type must have default() method defined.
                self.declare_local(local_decl.name.text.clone(), ty);
                Ok(ty)
            }
            // Expression must be assignable to the defined type.
            (Some(ty), Some(expr_ty)) => {
                // TODO: Upcasting to interfaces.
                if ty == expr_ty {
                    self.declare_local(local_decl.name.text.clone(), ty);
                    Ok(ty)
                } else {
                    typecheck_err(format!("mismatched types; expected {:?}, found {:?}", ty, expr_ty)).into()
                }
            }
        }
    }

    /// Type check the given expression node.
    pub fn check_expr(&mut self, expr: &Expr) -> Result<TypeId> {
        match expr {
            Expr::Name(_) => todo!(),
            Expr::Binary(binary_expr) => self.check_binary_expr(binary_expr),
            Expr::Lit(literal) => Ok(literal.type_id()),
            Expr::Func(_) => todo!(),
            Expr::Call(_) => todo!(),
        }
    }

    fn check_binary_expr(&mut self, binary_expr: &BinaryExpr) -> Result<TypeId> {
        let lhs_ty = self.check_expr(&binary_expr.lhs)?;
        let rhs_ty = self.check_expr(&binary_expr.rhs)?;

        match (lhs_ty, binary_expr.op, rhs_ty) {
            (TYPE_INT_ID, _, TYPE_INT_ID) => Ok(TYPE_INT_ID),
            (TYPE_FLOAT_ID, _, TYPE_FLOAT_ID) => Ok(TYPE_FLOAT_ID),
            (TYPE_STRING_ID, BinaryOp::Add, TYPE_STRING_ID) => Ok(TYPE_STRING_ID),
            _ => typecheck_err(format!("{:?} {:?} {:?}", lhs_ty, binary_expr.op, rhs_ty)).into(),
        }
    }

    /// Declare a local variable in the current scope.
    fn declare_local(&mut self, name: String, ty: TypeId) {
        match self.scope.locals.iter().position(|l| l.name == name) {
            // Existing local is shadowed.
            Some(index) => {
                self.scope.locals[index] = Local { name, ty };
            }
            // New variable declared.
            None => {
                self.scope.locals.push(Local { name, ty });
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_typecheck_block() {
        let block = Block {
            ty: TYPE_VOID_ID,
            stmts: vec![
                // Type inference case
                Stmt::Local(Box::new(LocalDecl {
                    name: Ident::from_string("x"),
                    ty: None,
                    rhs: Some(Expr::Binary(Box::new(BinaryExpr {
                        op: BinaryOp::Add,
                        lhs: Expr::Lit(Box::new(Literal::Num(Number::Int(7)))),
                        rhs: Expr::Lit(Box::new(Literal::Num(Number::Int(11)))),
                    }))),
                })),
                // Both type and initial value
                Stmt::Local(Box::new(LocalDecl {
                    name: Ident::from_string("x"),
                    ty: Some(TypeDef::Alias(TypeName {
                        text: Ident::from_string("Int"),
                    })),
                    rhs: Some(Expr::Lit(Box::new(Literal::Num(Number::Int(42))))),
                })),
            ],
        };

        let mut typechecker = TypeChecker::new();

        typechecker.check_block(&block).expect("typechecking block");
    }

    #[test]
    fn test_typecheck_expression() {
        let expr = Expr::Binary(Box::new(BinaryExpr {
            op: BinaryOp::Add,
            lhs: Expr::Lit(Box::new(Literal::Num(Number::Int(1)))),
            rhs: Expr::Lit(Box::new(Literal::Num(Number::Float(2.0)))),
        }));

        let mut typechecker = TypeChecker::new();

        assert!(typechecker.check_expr(&expr).is_err());
    }
}

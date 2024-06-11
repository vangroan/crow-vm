//! Type system.

/// Initialiase the table of types, with the built in types in their proper positions.
pub fn init_type_table() -> Vec<Type> {
    vec![Type::Void, Type::Int, Type::Float, Type::String]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeId(pub(crate) u32);

impl Default for TypeId {
    fn default() -> Self {
        TYPE_VOID_ID
    }
}

// Important: The values of the built-in type identifiers must
//            match the index in `init_type_table()`.
pub const TYPE_VOID_ID: TypeId = TypeId(0);
pub const TYPE_INT_ID: TypeId = TypeId(1);
pub const TYPE_FLOAT_ID: TypeId = TypeId(2);
pub const TYPE_STRING_ID: TypeId = TypeId(3);

#[derive(Debug, PartialEq, Eq)]
pub enum Type {
    /// The "unit" type returned by functions with no return value.
    ///
    /// It is a type error to assign [`Type::Void`] to a variable.
    /// A block or function that returns void must have its value discarded.
    Void,
    Int,
    Float,
    String,
    /// List of types for when multiple values are returned from a block,
    /// or function.
    Tuple(Vec<TypeId>),
    Array(TypeId),
    /// Hash table.
    Table(TypeId, TypeId),
    /// Type of both the [`crate::object::Closure`] value and [`crate::object::Func`]` prototype.
    Func {
        args: Vec<TypeId>,
        retunr_: TypeId,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    /// Ensure that the builtin type identifiers match their indices.
    #[test]
    fn test_type_index() {
        let types = init_type_table();
        assert_eq!(types[TYPE_VOID_ID.0 as usize], Type::Void);
        assert_eq!(types[TYPE_INT_ID.0 as usize], Type::Int);
        assert_eq!(types[TYPE_FLOAT_ID.0 as usize], Type::Float);
        assert_eq!(types[TYPE_STRING_ID.0 as usize], Type::String);
    }
}

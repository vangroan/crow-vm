use std::ptr::NonNull;

/// Pointer to an object.
#[derive(Debug, Clone, Copy)]
pub struct ObjPtr(pub(crate) NonNull<Object>);

#[derive(Debug, Clone, Copy)]
pub enum ObjectKind {
    String,
    Function,
}

pub struct Object {
    kind: ObjectKind,
}

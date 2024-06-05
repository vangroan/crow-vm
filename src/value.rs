use std::fmt::{self, Formatter};

use crate::object::ObjPtr;

/// Value is a typed, safe value.
#[derive(Debug, Clone, Copy)]
pub enum Value {
    Int(i64),
    UInt(u64),
    Float(f64),
    Object(ObjPtr),
}

/// Slot is an untyped, unsafe value.
#[derive(Clone, Copy)]
pub union Slot {
    pub(crate) int: i64,
    pub(crate) uint: u64,
    pub(crate) float: f64,
    pub(crate) object: ObjPtr,
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let data = unsafe { self.uint };
        write!(f, "Slot{{ 0x{data:x} }}")
    }
}

use std::fmt::{self, Formatter};
use std::rc::Rc;

use crate::func::Func;
use crate::object::ObjPtr;

/// Value is a typed, safe value.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    UInt(u64),
    Float(f64),
    Object(Obj),
}

impl Value {
    pub fn new_bool(val: bool) -> Self {
        Value::Int(if val { 1 } else { 0 })
    }

    pub fn as_int(&self) -> Option<i64> {
        match *self {
            Value::Int(val) => Some(val),
            _ => None,
        }
    }

    pub fn new_func(func: Rc<Func>) -> Self {
        Value::Object(Obj::Func(func))
    }

    pub fn as_func(&self) -> Option<&Rc<Func>> {
        match self {
            Value::Object(Obj::Func(ref func_rc)) => Some(func_rc),
            _ => None,
        }
    }

    pub fn to_func(&self) -> Option<Rc<Func>> {
        match self {
            Value::Object(Obj::Func(func_rc)) => Some(func_rc.clone()),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum Obj {
    Func(Rc<Func>),
}

impl fmt::Debug for Obj {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::Func(rc) => write!(f, "Func(0x{:?})", Rc::as_ptr(rc)),
        }
    }
}

/// Slot is an untyped, unsafe value.
#[derive(Clone, Copy)]
pub union Slot {
    pub(crate) int: i64,
    pub(crate) uint: u64,
    pub(crate) float: f64,
    pub(crate) object: ObjPtr,
}

impl Slot {
    /// A slot that's considered empty.
    pub(crate) const fn empty() -> Self {
        Slot { uint: 0 }
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let data = unsafe { self.uint };
        write!(f, "Slot{{ 0x{data:x} }}")
    }
}

use std::fmt::{self, Formatter};
use std::ptr::NonNull;
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
    pub fn from_bool(val: bool) -> Self {
        Value::Int(if val { 1 } else { 0 })
    }

    pub fn as_int(&self) -> Option<i64> {
        match *self {
            Value::Int(val) => Some(val),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match *self {
            Value::Float(val) => Some(val),
            _ => None,
        }
    }

    pub fn from_func(func: Rc<Func>) -> Self {
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

    pub(crate) unsafe fn from_func(func: Rc<Func>) -> Self {
        Slot {
            object: ObjPtr(NonNull::new(Rc::into_raw(func) as *mut _).unwrap()),
        }
    }

    pub(crate) unsafe fn into_func(self) -> Rc<Func> {
        todo!()
    }
}

impl fmt::Debug for Slot {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let data = unsafe { self.uint };
        write!(f, "Slot{{ 0x{data:x} }}")
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::errors::Result;
    use crate::{
        func::Constants,
        op::{Arg24, Op},
    };

    /// Experimental Miri test
    #[test]
    fn test_slot() -> Result<()> {
        let func = Rc::new(Func {
            code: Box::new([
                Op::PushIntIn(Arg24::from_i64(7)?),
                Op::PushIntIn(Arg24::from_i64(11)?),
                Op::Int_Add,
                Op::End,
            ]),
            stack_size: 3,
            is_varg: true,
            constants: Constants {
                ints: Box::new([]),
                floats: Box::new([]),
                strings: Box::new([]),
                funcs: Box::new([]),
            },
        });

        // let slot = unsafe {
        //     Slot::from_func(func)
        // };

        // println!("{:?}", unsafe { slot.float });

        Ok(())
    }
}

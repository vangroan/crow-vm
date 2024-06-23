use std::fmt::{self, Formatter};
use std::ptr::NonNull;
use std::rc::Rc;

use crate::handle::Handle;
use crate::object::*;

/// Value is a typed, safe value.
#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    UInt(u64),
    Float(f64),
    Object(Object),
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
        Value::Object(Object::Func(func))
    }

    pub fn as_func(&self) -> Option<&Rc<Func>> {
        match self {
            Value::Object(Object::Func(ref func_rc)) => Some(func_rc),
            _ => None,
        }
    }

    pub fn to_func(self) -> Option<Rc<Func>> {
        match self {
            Value::Object(Object::Func(func_rc)) => Some(func_rc.clone()),
            _ => None,
        }
    }

    pub fn as_table(&self) -> Option<&Handle<Table>> {
        match self {
            Value::Object(Object::Table(ref table_handle)) => Some(table_handle),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<&Rc<CrowStr>> {
        match self {
            Value::Object(Object::String(ref table_handle)) => Some(table_handle),
            _ => None,
        }
    }

    pub fn from_closure(closure: Rc<Closure>) -> Self {
        Value::Object(Object::Closure(closure))
    }

    pub fn as_closure(&self) -> Option<&Rc<Closure>> {
        match self {
            Value::Object(Object::Closure(ref rc)) => Some(rc),
            _ => None,
        }
    }
}

/// TODO: Unsafe memory management.
#[derive(Clone, Copy)]
pub struct ObjPtr(NonNull<()>);

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
        object::Constants,
        op::{Arg24, Op},
    };

    #[test]
    fn test_value_size() {
        assert!(
            std::mem::size_of::<Value>() <= std::mem::size_of::<usize>() * 2,
            "Value should be at most two machine words"
        );
    }

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
            up_values: Box::new([]),
        });

        // let slot = unsafe {
        //     Slot::from_func(func)
        // };

        // println!("{:?}", unsafe { slot.float });

        Ok(())
    }
}

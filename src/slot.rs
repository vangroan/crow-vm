use std::rc::Rc;

use crate::func::Func;
use crate::string::Str;

/// Value in the operand stack.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Slot(pub(crate) u64);

impl Slot {
    #[inline(always)]
    pub fn from_bool(b: bool) -> Self {
        Self(if b { 1 } else { 0 })
    }

    #[inline(always)]
    pub fn from_int(int: i64) -> Self {
        Self(int as u64)
    }

    #[inline(always)]
    pub fn from_float(float: f64) -> Self {
        Self(float.to_bits())
    }

    #[inline(always)]
    pub fn from_string(string: Str) -> Self {
        unsafe { Self(string.as_ptr() as _) }
    }
    
    #[inline(always)]
    pub fn from_func(func: Rc<Func>) -> Self {
        unsafe { Self(Rc::into_raw(func) as _) }
    }

    #[inline(always)]
    pub fn as_bool(self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub fn as_int(self) -> i64 {
        self.0 as i64
    }

    #[inline(always)]
    pub fn as_float(self) -> f64 {
        f64::from_bits(self.0)
    }

    #[inline(always)]
    pub fn as_string(self) -> Str {
        unsafe { Str::from_ptr(self.0 as *const String) }
    }

    #[inline(always)]
    pub fn as_func(self) -> Rc<Func> {
        unsafe { Rc::from_raw(self.0 as *const Func) }
    }
}



pub trait IntoSlot {
    fn is_object(&self) -> bool;
    fn into_slot(self) -> Slot;
}

impl IntoSlot for Rc<Func> {
    fn is_object(&self) -> bool {
        true
    }

    fn into_slot(self) -> Slot {
        Slot::from_func(self)
    }
}

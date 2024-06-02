use crate::string::Str;

/// Value in the operand stack.
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Slot(pub(crate) u64);

impl Slot {
    #[inline(always)]
    fn from_int(self, int: i64) -> Self {
        Self(int as u64)
    }

    #[inline(always)]
    fn from_float(self, float: f64) -> Self {
        Self(float.to_bits())
    }

    #[inline(always)]
    fn from_string(self, string: Str) -> Self {
        unsafe { Self(string.as_ptr() as _) }
    }

    #[inline(always)]
    fn as_int(self) -> i64 {
        self.0 as i64
    }

    #[inline(always)]
    fn as_float(self) -> f64 {
        f64::from_bits(self.0)
    }

    #[inline(always)]
    fn as_string(self) -> Str {
        unsafe { Str::from_ptr(self.0 as *const String) }
    }
}

use crate::errors::{runtime_err, Error, Result};
use crate::limits::*;

/// Bytecode instruction.
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Op {
    /// Does nothing. Only the instruction pointer is creased.
    NoOp,

    /// Remove and discard the top value from the stack.
    Pop(Arg24),
    End,
    Return {
        /// Actual number of result values returned by the callee.
        results: u8,
    },

    /// Call either a script or native function.
    Call {
        /// Stack base relative to the caller's stack base.
        base: u16,
        /// Number of result values the caller expects to be returned
        /// from the callee.
        results: u8,
    },

    /// Copy multiple values from the stack offset to the top.
    Load {
        offset: u16,
        len: u8,
    },
    /// Copy multiple values from the top of the stack to the given offset.
    Store {
        offset: u16,
        len: u8,
    },

    SetLocal {
        slot: u16,
    },
    GetLocal {
        slot: u16,
    },

    SetGlobal {
        string: u16,
    },
    GetGlobal {
        string: u16,
    },

    /// Push an inlined integer value onto the stack.
    PushIntIn(Arg24),
    /// Push an integer constant onto the stack.
    PushInt(Arg24),
    PushFloat(Arg24),
    PushString(Arg24),
    PushFunc(Arg24),

    // Integer arithmetic.
    Int_Neq,
    Int_Add,
    Int_Sub,
    Int_Mul,
    Int_Div,
    Int_Mod,

    // Integer Comparison
    Int_Ne,
    Int_Eq,
    Int_Lt,
    Int_Le,
    Int_Gt,
    Int_Ge,

    // Float arithmetic
    Float_Neq,
    Float_Add,
    Float_Sub,
    Float_Mul,
    Float_Div,
    Float_Mod,

    // Float Comparison
    Float_Ne,
    Float_Eq,
    Float_Lt,
    Float_Le,
    Float_Gt,
    Float_Ge,

    // String operations
    Str_Concat,
    Str_Slice,

    // Jumps
    JumpNe {
        addr: Arg24,
    },
    JumpEq {
        addr: Arg24,
    },
    JumpLt {
        addr: Arg24,
    },
    JumpLe {
        addr: Arg24,
    },
    JumpGt {
        addr: Arg24,
    },
    JumpGe {
        addr: Arg24,
    },
    JumpZero {
        addr: Arg24,
    },
    Jump {
        addr: Arg24,
    },
}

impl Op {
    pub fn stack_effect(&self) -> isize {
        todo!()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Arg24([u8; 3]);

impl Arg24 {
    #[inline(always)]
    pub fn into_i64(self) -> i64 {
        let [a, b, c] = self.0;
        // Place the bytes into the most signifigant to shift
        // down and preseve the sign.
        i64::from_le_bytes([0, 0, 0, 0, 0, a, b, c]) >> 40
    }

    #[inline(always)]
    pub fn into_u32(self) -> u32 {
        let [a, b, c] = self.0;
        u32::from_le_bytes([a, b, c, 0])
    }

    #[inline(always)]
    pub fn into_usize(self) -> usize {
        let [a, b, c] = self.0;
        u32::from_le_bytes([a, b, c, 0]) as usize
    }

    #[inline(always)]
    pub fn from_i64(value: i64) -> Result<Self> {
        if value >= MAX_ARG_24 {
            Err(runtime_err("value is too large to fit in 24 bits"))
        } else if value <= MIN_ARG24 {
            Err(runtime_err("value is too small to fit in 24 bits"))
        } else {
            let [a, b, c, _, _, _, _, _] = value.to_le_bytes();
            Ok(Self([a, b, c]))
        }
    }

    #[inline(always)]
    pub fn from_u32(value: u32) -> Result<Self> {
        let [a, b, c, _] = value.to_le_bytes();
        Ok(Self([a, b, c]))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_op_size() {
        assert!(
            std::mem::size_of::<Op>() <= 4,
            "instruction must by at most 32-bits"
        );
    }

    #[test]
    fn test_arg24() {
        assert_eq!(Arg24::from_i64(1).unwrap().0, [1, 0, 0]);
        assert_eq!(Arg24::from_i64(1).unwrap().into_i64(), 1);
    }
}

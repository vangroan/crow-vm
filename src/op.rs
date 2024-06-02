/// Bytecode instruction.
#[derive(Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum Op {
    /// Does nothing. Only the instruction pointer is creased.
    NoOp,

    /// Remove and discard the top value from the stack.
    Pop,
    Return {
        /// Number of stack values returned by the callee.
        results: u8,
    },

    Call {
        /// Stack base relative to the caller's stack base.
        base: u16,
        results: u8,
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

    PushInt,
    PushFloat,
    PushString,

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
    JumpNe,
    JumpEq,
    JumpLt,
    JumpLe,
    JumpGt,
    JumpGe,
    Jump,
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
}

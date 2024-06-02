use std::rc::Rc;

use crate::errors::{runtime_err, Result};
use crate::func::Func;
use crate::op::Op;
use crate::slot::Slot;

pub struct Vm {
    /// Operand stack.
    stack: Vec<Slot>,

    /// Callstack.
    calls: Vec<CallFrame>,
}

struct CallFrame {
    /// Instruction pointer.
    ip: usize,
    /// Stack base where the frame's local variables and temporary value start.
    base: usize,
    /// Function prototype that this frame is executing.
    func: Rc<Func>,
}

#[derive(Debug)]
enum FrameAction {
    /// Return from the child frame to the parent frame.
    Return,

    /// Call a new function.
    Call {},
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            calls: vec![],
        }
    }

    pub fn run(&mut self, env: (), func: Rc<Func>) -> Result<()> {
        run_interpreter(self, func)
    }
}

impl CallFrame {
    fn new(func: Rc<Func>) -> Self {
        Self {
            ip: 0,
            base: 0,
            func,
        }
    }
}

fn run_interpreter(vm: &mut Vm, func: Rc<Func>) -> Result<()> {
    let mut frame = CallFrame::new(func);

    loop {
        match run_op_loop(vm, &mut frame)? {
            FrameAction::Return => todo!(),
            FrameAction::Call {} => todo!(),
        }
    }
}

fn run_op_loop(vm: &mut Vm, frame: &mut CallFrame) -> Result<FrameAction> {
    loop {
        let op = frame.func.code[frame.ip];
        frame.ip += 1;

        match op {
            Op::NoOp => { /* Do nothing */ }
            Op::Pop => {
                vm.stack.pop();
            }
            Op::Return { results } => {
                todo!()
            }

            Op::Call { base, results } => {
                todo!()
            }

            Op::SetLocal { slot } => {
                vm.stack[slot as usize] = vm
                    .stack
                    .last()
                    .cloned()
                    .ok_or_else(|| runtime_err("stack underflow"))?;
            }
            Op::GetLocal { slot } => {
                vm.stack.push(vm.stack[slot as usize]);
            }

            Op::SetGlobal { string } => todo!(),
            Op::GetGlobal { string } => todo!(),
            Op::PushInt => todo!(),
            Op::PushFloat => todo!(),
            Op::PushString => todo!(),
            Op::Int_Neq => todo!(),
            Op::Int_Add => todo!(),
            Op::Int_Sub => todo!(),
            Op::Int_Mul => todo!(),
            Op::Int_Div => todo!(),
            Op::Int_Mod => todo!(),

            Op::Int_Ne => todo!(),
            Op::Int_Eq => todo!(),
            Op::Int_Lt => todo!(),
            Op::Int_Le => todo!(),
            Op::Int_Gt => todo!(),
            Op::Int_Ge => todo!(),

            Op::Float_Neq => todo!(),
            Op::Float_Add => todo!(),
            Op::Float_Sub => todo!(),
            Op::Float_Mul => todo!(),
            Op::Float_Div => todo!(),
            Op::Float_Mod => todo!(),

            Op::Float_Ne => todo!(),
            Op::Float_Eq => todo!(),
            Op::Float_Lt => todo!(),
            Op::Float_Le => todo!(),
            Op::Float_Gt => todo!(),
            Op::Float_Ge => todo!(),

            Op::Str_Concat => todo!(),
            Op::Str_Slice => todo!(),

            Op::JumpNe => todo!(),
            Op::JumpEq => todo!(),
            Op::JumpLt => todo!(),
            Op::JumpLe => todo!(),
            Op::JumpGt => todo!(),
            Op::JumpGe => todo!(),
            Op::Jump => todo!(),
        }
    }
}

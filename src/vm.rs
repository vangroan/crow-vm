use std::rc::Rc;

use crate::errors::{runtime_err, Error, Result};
use crate::func::Func;
use crate::op::Op;
use crate::slot::Slot;

pub struct Vm {
    /// Operand stack.
    pub(crate) stack: Vec<Slot>,

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
    Return { results: u8 },

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

    pub fn run(&mut self, _env: (), func: Rc<Func>) -> Result<()> {
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
            FrameAction::Return { .. } => {
                if vm.calls.is_empty() {
                    return Ok(());
                }
                todo!("calls and returns")
            }
            FrameAction::Call {} => todo!(),
        }
    }
}

fn err_stack_underflow() -> Error {
    runtime_err("stack underflow")
}

fn run_op_loop(vm: &mut Vm, frame: &mut CallFrame) -> Result<FrameAction> {
    loop {
        let op = frame
            .func
            .code
            .get(frame.ip)
            .cloned()
            .ok_or_else(|| runtime_err("instruction pointer out of bytecode bounds"))?;
        frame.ip += 1;

        println!("vm.stack -> {:?}", vm.stack);

        match op {
            Op::NoOp => { /* Do nothing */ }
            Op::Pop => {
                vm.stack.pop();
            }
            Op::End => return Ok(FrameAction::Return { results: 0 }),
            Op::Return { .. } => {
                todo!()
            }

            Op::Call { .. } => {
                todo!()
            }

            Op::Load{ offset, len } => {
                let index_0 = vm.stack.len();
                vm.stack.extend((0..len).map(|_| Slot(0)));

                let start_a = frame.base + offset as usize;
                let (a, b) = vm.stack.split_at_mut(start_a);

                for (x, y) in a.iter().zip(b.iter_mut()) {
                    *y = *x;
                }
            }
            Op::Store{ offset, len } => {}

            Op::SetLocal { slot } => {
                vm.stack[slot as usize] =
                    vm.stack.last().cloned().ok_or_else(err_stack_underflow)?;
            }
            Op::GetLocal { slot } => {
                vm.stack.push(vm.stack[slot as usize]);
            }

            Op::SetGlobal { .. } => todo!(),
            Op::GetGlobal { .. } => todo!(),

            Op::PushInt(value) => {
                vm.stack.push(Slot::from_int(value.into_i64()));
            }
            Op::PushFloat => todo!(),
            Op::PushString => todo!(),
            Op::Int_Neq => {
                let a = vm.stack[frame.ip].as_int();
                vm.stack[frame.ip] = Slot::from_int(-a);
            }
            Op::Int_Add => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_int(a + b));
            }
            Op::Int_Sub => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_int(a - b));
            }
            Op::Int_Mul => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_int(a * b));
            }
            Op::Int_Div => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_int(a / b));
            }
            Op::Int_Mod => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_int(a % b));
            }

            Op::Int_Ne => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_bool(a != b));
            }
            Op::Int_Eq => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_bool(a == b));
            }
            Op::Int_Lt => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_bool(a < b));
            }
            Op::Int_Le => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_bool(a <= b));
            }
            Op::Int_Gt => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_bool(a > b));
            }
            Op::Int_Ge => {
                let b = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                vm.stack.push(Slot::from_bool(a >= b));
            }

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

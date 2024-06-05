use std::rc::Rc;

use crate::errors::{runtime_err, Error, Result};
use crate::func::Func;
use crate::op::Op;
use crate::slot::{IntoSlot, Slot};

pub struct Vm {
    /// Operand stack.
    pub(crate) stack: Vec<Slot>,

    /// Annotation metadata to mark stack slots as pointers.
    pub pointers: Vec<bool>,

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
    Call { base: u16, results: u8 },
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            pointers: vec![],
            calls: vec![],
        }
    }

    pub fn run(&mut self, _env: (), func: Rc<Func>) -> Result<()> {
        run_interpreter(self, func)
    }

    fn is_marked(&self, slot: usize) -> bool {
        self.pointers.get(slot).cloned().unwrap_or(false)
    }

    fn mark_pointer(&mut self, index: usize) {
        self.pointers[index] = true;
    }

    fn unmark_pointer(&mut self, index: usize) {
        self.pointers[index] = false;
    }

    fn grow_pointer_stack(&mut self, extra_space: usize) {
        let required = self.stack.len() + extra_space;
        if self.pointers.len() < required {
            self.pointers
                .extend((0..(required - self.pointers.len())).map(|_| false));
        }
    }

    fn pop(&mut self) -> Result<Slot> {
        match self.stack.pop() {
            Some(slot) => {
                self.pointers.pop();
                Ok(slot)
            }
            None => err_stack_underflow().into(),
        }
    }

    #[inline(always)]
    fn push<T: IntoSlot>(&mut self, value: T) {
        if value.is_object() {
            let index = self.stack.len();
            self.mark_pointer(index);
        }
        self.stack.push(value.into_slot());
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

/// Interpreter entry point.
fn run_interpreter(vm: &mut Vm, func: Rc<Func>) -> Result<()> {
    // Prepare stack space.
    vm.grow_pointer_stack(func.stack_size as usize);

    let mut frame = CallFrame::new(func.clone());

    // To keep consistent with the calling convention,
    // place the callable on the stack.
    // FIXME: Memory management to ensure this Rc<Func> isn't leaked.
    vm.push(func);

    loop {
        match run_op_loop(vm, &mut frame)? {
            FrameAction::Return { results } => {
                if vm.calls.is_empty() {
                    for _ in 0..results {
                        println!("return: {:?}", vm.stack.pop());
                    }
                    vm.stack.truncate(frame.base);
                    return Ok(());
                }
                todo!("calls and returns")
            }
            FrameAction::Call { base, .. } => {
                // base is relative to the caller's base.
                let callee_base = frame.base + base as usize;

                // A callable is expected at the callee's stack.
                if !vm.is_marked(callee_base) {
                    return runtime_err("callable is not a pointer").into();
                }

                let func = vm.stack[callee_base].as_func();

                vm.grow_pointer_stack(func.stack_size as usize);

                let new_frame = CallFrame {
                    ip: 0,
                    base: callee_base,
                    func,
                };

                vm.calls.push(std::mem::replace(&mut frame, new_frame));
            }
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
        println!("            {:?}", vm.pointers);

        match op {
            Op::NoOp => { /* Do nothing */ }
            Op::Pop => {
                vm.stack.pop();
            }
            Op::End => return Ok(FrameAction::Return { results: 0 }),
            Op::Return { results } => return Ok(FrameAction::Return { results }),

            Op::Call { base, results } => {
                todo!()
            }

            Op::Load { offset, len } => {
                todo!()
            }
            Op::Store { offset, len } => {}

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

            Op::JumpNe { .. } => todo!(),
            Op::JumpEq { .. } => todo!(),
            Op::JumpLt { .. } => todo!(),
            Op::JumpLe { .. } => todo!(),
            Op::JumpGt { .. } => todo!(),
            Op::JumpGe { .. } => todo!(),
            Op::JumpZero { addr } => {
                let a = vm.stack.pop().ok_or_else(err_stack_underflow)?.as_int();
                if a == 0 {
                    frame.ip = (frame.ip as i64 + addr.into_i64()) as usize;
                }
            }
            Op::Jump { addr } => {
                frame.ip = (frame.ip as i64 + addr.into_i64()) as usize;
            }
        }
    }
}

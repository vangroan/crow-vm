use std::rc::Rc;

use crate::errors::{runtime_err, Error, Result};
use crate::func::Func;
use crate::op::Op;
use crate::value::Value;

pub struct Vm {
    /// Operand stack.
    pub(crate) stack: Vec<Value>,

    /// Callstack.
    calls: Vec<CallFrame>,
}

struct CallFrame {
    /// Instruction pointer.
    ip: usize,
    /// Pointer to the top of the stack, relative to it's local base.
    top: usize,
    /// Stack base where the frame's local variables and temporary value start.
    base: usize,
    /// The number of resulting values the caller expects from the callee.
    results: usize,
    /// Function prototype that this frame is executing.
    func: Rc<Func>,
}

#[derive(Debug)]
enum FrameAction {
    /// Return from the child frame to the parent frame.
    Return { start: usize, count: u8 },

    /// Call a new function.
    Call { base: usize, results: u8 },
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

    fn grow_stack(&mut self, additional: usize) {
        self.stack.extend((0..additional).map(|_| Value::Int(0)))
    }
}

impl CallFrame {
    fn new(func: Rc<Func>) -> Self {
        Self {
            ip: 0,
            top: 0,
            base: 0,
            results: 0,
            func,
        }
    }
}

/// Interpreter entry point.
fn run_interpreter(vm: &mut Vm, func: Rc<Func>) -> Result<()> {
    // FIXME: Memory management to ensure this Rc<Func> isn't leaked.
    let mut frame = CallFrame::new(func.clone());

    vm.stack.push(Value::new_func(frame.func.clone()));

    loop {
        match run_op_loop(vm, &mut frame)? {
            FrameAction::Return { start, count } => {
                println!(
                    "return: frame.base->{}, slot->{:?}",
                    frame.base, vm.stack[frame.base]
                );

                // Drop callable to decrement reference count.
                // let _ = vm.stack[frame.base].as_func();

                if vm.calls.is_empty() {
                    for _ in 0..count {
                        println!("return: {:?}", vm.stack.pop());
                    }
                    vm.stack.truncate(frame.base);
                    return Ok(());
                }

                // Copy the multiple returns to the base of the stack.
                // Erase the callable.
                //
                // The caller may be expecting less or more results
                // than what the callee is actually returning.
                if frame.results > (count as usize) {
                    return runtime_err(format!(
                        "caller expected {} results, but callee only returned {count}",
                        frame.results
                    ))
                    .into();
                }

                // The callee may return more results, but the caller can just discard them.
                let results = frame.results.min(count as usize);

                let stack = &mut vm.stack[frame.base..frame.base + frame.func.stack_size as usize];
                let result_span = 0..results;

                // Copy the callee's results to its base, so they're available to the callee.
                for offset in result_span {
                    stack[offset] = stack[start as usize + offset].clone();
                }

                vm.stack.truncate(frame.base + results);
                println!("vm.stack (after truncate) -> {:?}", vm.stack);

                frame = vm.calls.pop().unwrap();
            }
            FrameAction::Call { base, results } => {
                // base is relative to the caller's base.
                let callee_base = frame.base + base as usize;
                let slot = vm.stack[callee_base].clone();

                println!("call: frame.base->{}, callee_base->{:?}", frame.base, slot);

                let new_frame = CallFrame {
                    ip: 0,
                    top: 1,
                    base: callee_base,
                    results: results as usize,
                    func: vm.stack[callee_base]
                        .to_func()
                        .ok_or_else(err_func_expected)?,
                };

                vm.calls.push(std::mem::replace(&mut frame, new_frame));
            }
        }
    }
}

fn err_stack_underflow() -> Error {
    runtime_err("stack underflow")
}

fn err_func_expected() -> Error {
    runtime_err("function value expected")
}

fn err_int_expected() -> Error {
    runtime_err("integer value expected")
}

fn run_op_loop(vm: &mut Vm, frame: &mut CallFrame) -> Result<FrameAction> {
    // let Vm { stack: whole_stack, .. } = vm;

    // Slice has a fixed size which allows the compiler some more optimisations.
    // let stack = &whole_stack[frame.base..];

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
            Op::Pop(n) => {
                for _ in 0..n.into_u32() {
                    vm.stack.pop();
                }
            }
            Op::End => return Ok(FrameAction::Return { start: 0, count: 0 }),
            Op::Return { results: count } => {
                // Top values on stack are considered the return values.
                let start = vm.stack.len() - frame.base - count as usize;
                return Ok(FrameAction::Return { start, count });
            }

            Op::Call { base, results } => {
                return Ok(FrameAction::Call {
                    base: frame.base + base as usize,
                    results,
                })
            }

            Op::Load { .. } => {
                todo!()
            }
            Op::Store { .. } => {
                todo!()
            }

            Op::SetLocal { slot } => {
                vm.stack[slot as usize] =
                    vm.stack.last().cloned().ok_or_else(err_stack_underflow)?;
            }
            Op::GetLocal { slot } => {
                vm.stack.push(vm.stack[slot as usize].clone());
            }

            Op::SetGlobal { .. } => todo!(),
            Op::GetGlobal { .. } => todo!(),

            Op::PushIntIn(value) => {
                vm.stack.push(Value::Int(value.into_i64()));
            }
            Op::PushInt(_const_id) => todo!(),
            Op::PushFloat(_const_id) => todo!(),
            Op::PushString(_const_id) => todo!(),
            Op::PushFunc(const_id) => {
                let func = frame
                    .func
                    .constants
                    .funcs
                    .get(const_id.into_usize())
                    .ok_or_else(|| {
                        runtime_err(format!(
                            "no function found at constant {}",
                            const_id.into_usize()
                        ))
                    })?;
                vm.stack.push(Value::new_func(func.clone()));
            }
            Op::Int_Neq => {
                let a = vm.stack[frame.ip].as_int().ok_or_else(err_int_expected)?;
                vm.stack[frame.ip] = Value::Int(-a);
            }
            Op::Int_Add => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::Int(a + b));
            }
            Op::Int_Sub => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::Int(a - b));
            }
            Op::Int_Mul => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::Int(a * b));
            }
            Op::Int_Div => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::Int(a / b));
            }
            Op::Int_Mod => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::Int(a % b));
            }

            Op::Int_Ne => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::new_bool(a != b));
            }
            Op::Int_Eq => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::new_bool(a == b));
            }
            Op::Int_Lt => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::new_bool(a < b));
            }
            Op::Int_Le => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::new_bool(a <= b));
            }
            Op::Int_Gt => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::new_bool(a > b));
            }
            Op::Int_Ge => {
                let b = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
                vm.stack.push(Value::new_bool(a >= b));
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
                let a = vm
                    .stack
                    .pop()
                    .ok_or_else(err_stack_underflow)?
                    .as_int()
                    .ok_or_else(err_int_expected)?;
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

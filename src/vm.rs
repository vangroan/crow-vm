use std::rc::Rc;

use crate::errors::{runtime_err, Error, Result};
use crate::handle::Handle;
use crate::object::{Closure, Func, Object, UpValue, UpValueOrigin};
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
    /// The closure being executed by this frame.
    closure: Rc<Closure>,
    /// Function prototype that this frame is executing.
    func: Rc<Func>,
    /// These are the *open* up-values belonging to all closures created in this
    /// call frame that captured local variables belonging to this call frame.
    /// They are shared with each closures' heap space so that closing them
    /// reflects within those closures' when it escapes.
    ///
    /// Before this frame is popped off the call stack, all its captured locals must
    /// be copied into the up-values, and the up-values closed.
    up_values: Vec<Handle<UpValue>>,
}

#[derive(Debug)]
enum FrameAction {
    /// Return from the child frame to the parent frame.
    ///
    /// Start of results on start is absolute.
    Return { start: usize, count: u8 },

    /// Call a new function.
    ///
    /// Base of stack is absolute.
    Call { base: usize, results: u8 },
}

impl Vm {
    pub fn new() -> Self {
        Self {
            stack: vec![],
            calls: vec![],
        }
    }

    /// Execute a function constant.
    pub fn run_function(&mut self, _env: (), func: Rc<Func>) -> Result<()> {
        // All callables are wrapped in closures to simplify the VM loop.
        let closure = Rc::new(Closure::new(func));
        run_interpreter(self, closure)
    }

    fn grow_stack(&mut self, additional: usize) {
        self.stack.extend((0..additional).map(|_| Value::Int(0)))
    }

    fn pop_int(&mut self) -> Result<i64> {
        self.stack
            .pop()
            .ok_or_else(err_stack_underflow)?
            .as_int()
            .ok_or_else(err_int_expected)
    }

    fn pop2_int(&mut self) -> Result<[i64; 2]> {
        let b = self
            .stack
            .pop()
            .ok_or_else(err_stack_underflow)?
            .as_int()
            .ok_or_else(err_int_expected)?;
        let a = self
            .stack
            .pop()
            .ok_or_else(err_stack_underflow)?
            .as_int()
            .ok_or_else(err_int_expected)?;
        Ok([a, b])
    }

    fn pop2_float(&mut self) -> Result<[f64; 2]> {
        let b = self
            .stack
            .pop()
            .ok_or_else(err_stack_underflow)?
            .as_float()
            .ok_or_else(err_float_expected)?;
        let a = self
            .stack
            .pop()
            .ok_or_else(err_stack_underflow)?
            .as_float()
            .ok_or_else(err_float_expected)?;
        Ok([a, b])
    }
}

impl CallFrame {
    fn new(closure: Rc<Closure>) -> Self {
        Self {
            ip: 0,
            top: 0,
            base: 0,
            results: 0,
            func: closure.func.clone(),
            closure,
            up_values: Vec::new(),
        }
    }
}

impl CallFrame {
    fn jump(&mut self, offset: i64) {
        println!("      jump {:04} -> {:04}", self.ip, self.ip as i64 + offset);
        self.ip = (self.ip as i64 + offset) as usize;
    }
}

/// Interpreter entry point.
fn run_interpreter(vm: &mut Vm, closure: Rc<Closure>) -> Result<()> {
    // FIXME: Memory management to ensure this Rc<Closure> isn't leaked.
    let mut frame = CallFrame::new(closure.clone());

    vm.stack.push(Value::from_closure(frame.closure.clone()));

    loop {
        match run_op_loop(vm, &mut frame)? {
            FrameAction::Return { start, count } => {
                println!("return: frame.base->{}, slot->{:?}", frame.base, vm.stack[frame.base]);

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
                // Erasing the callable.
                //
                // The caller may be expecting more results
                // than what the callee is actually returning.
                if frame.results > (count as usize) {
                    return runtime_err(format!(
                        "caller expected {} results, but callee only returned {count}",
                        frame.results
                    ))
                    .into();
                }

                // The callee may return more results, but the caller could just discard them.
                let result_count = frame.results.min(count as usize);

                // Slice the stack to the callee's span so it's easier to work with.
                let stack = &mut vm.stack[frame.base..frame.base + frame.func.stack_size as usize];

                // This overflow can happen if the bytecode is malformed.
                // (Result instruction returned wrong count)
                if start + result_count > stack.len() {
                    return runtime_err("returned results overflow stack").into();
                }

                // Copy the callee's results to its base, so they're available to the caller.
                for offset in 0..result_count {
                    stack[offset] = stack[start as usize + offset].clone();
                }

                vm.stack.truncate(frame.base + result_count);
                println!("vm.stack (after truncate) -> {:?}", vm.stack);

                frame = vm.calls.pop().unwrap();
            }
            FrameAction::Call {
                base: callee_base,
                results,
            } => {
                // base is relative to the caller's base.
                let slot = vm.stack[callee_base].clone();

                println!("call: frame.base->{}, callee_base->{:?}", frame.base, slot);

                let closure = vm.stack[callee_base]
                    .as_closure()
                    .cloned()
                    .ok_or_else(err_closure_expected)?;

                let new_frame = CallFrame {
                    ip: 0,
                    top: 1,
                    base: callee_base,
                    results: results as usize,
                    func: closure.func.clone(),
                    closure,
                    up_values: Vec::new(),
                };

                vm.calls.push(std::mem::replace(&mut frame, new_frame));
            }
        }
    }
}

fn err_const_notfound() -> Error {
    runtime_err("constant not found")
}

fn err_upvalue_notfound() -> Error {
    runtime_err("up-value not found")
}

fn err_stack_underflow() -> Error {
    runtime_err("stack underflow")
}

fn err_func_expected() -> Error {
    runtime_err("function value expected")
}

fn err_closure_expected() -> Error {
    runtime_err("closure value expected")
}

fn err_int_expected() -> Error {
    runtime_err("integer value expected")
}

fn err_float_expected() -> Error {
    runtime_err("float value expected")
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

        println!("      {:?}", vm.stack);
        println!("{:04} : {:?}", frame.ip, op);

        match op {
            Op::NoOp => { /* Do nothing */ }
            Op::Pop(n) => {
                for _ in 0..n.into_u32() {
                    vm.stack.pop();
                }
            }
            Op::End => return Ok(FrameAction::Return { start: 0, count: 0 }),
            Op::Return { results: count } => {
                // Close up-values.
                //
                // This frame is about the go out of scope, so any captured
                // local variables must be preserved on the heap.
                for up_value_handle in frame.up_values.drain(..) {
                    let up_value = &mut *up_value_handle.borrow_mut();
                    if let UpValue::Open(stack_offset) = up_value {
                        let value = vm.stack[*stack_offset].clone();
                        up_value.close(value);
                    }
                }

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
                vm.stack[frame.base + slot as usize] = vm.stack.last().cloned().ok_or_else(err_stack_underflow)?;
            }
            Op::GetLocal { slot } => {
                vm.stack.push(vm.stack[frame.base + slot as usize].clone());
            }

            Op::SetUpValue { upvalue_id } => {
                let value = vm.stack.pop().ok_or_else(err_stack_underflow)?;

                match &mut *frame
                    .closure
                    .up_values
                    .borrow_mut()
                    .get(upvalue_id as usize)
                    .ok_or_else(err_upvalue_notfound)?
                    .borrow_mut()
                {
                    UpValue::Open(stack_offset) => {
                        vm.stack[*stack_offset] = value;
                    }
                    UpValue::Closed(upvalue) => {
                        *upvalue = value;
                    }
                }
            }
            Op::GetUpValue { upvalue_id } => {
                match &*frame
                    .closure
                    .up_values
                    .borrow()
                    .get(upvalue_id as usize)
                    .ok_or_else(err_upvalue_notfound)?
                    .borrow()
                {
                    UpValue::Open(stack_offset) => {
                        vm.stack.push(vm.stack[*stack_offset].clone());
                    }
                    UpValue::Closed(upvalue) => {
                        vm.stack.push(upvalue.clone());
                    }
                }
            }

            Op::SetGlobal { .. } => todo!(),
            Op::GetGlobal { .. } => todo!(),

            Op::PushIntIn(value) => {
                vm.stack.push(Value::Int(value.into_i64()));
            }
            Op::PushInt(const_id) => {
                let x =
                    *frame.func.constants.ints.get(const_id.into_usize()).ok_or_else(|| {
                        runtime_err(format!("no integer constant defined: {}", const_id.into_usize()))
                    })?;
                vm.stack.push(Value::Int(x));
            }
            Op::PushFloat(_const_id) => todo!(),
            Op::PushString(_const_id) => todo!(),
            Op::PushFunc(const_id) => {
                let func =
                    frame.func.constants.funcs.get(const_id.into_usize()).ok_or_else(|| {
                        runtime_err(format!("no function found at constant {}", const_id.into_usize()))
                    })?;
                vm.stack.push(Value::from_func(func.clone()));
            }
            Op::CaptureValue(_) => todo!(),
            Op::CreateClosure { func_id } => {
                let func = frame
                    .func
                    .constants
                    .funcs
                    .get(func_id.into_usize())
                    .cloned()
                    .ok_or_else(err_const_notfound)?;
                let mut upvalues = Vec::new();
                let parent_upvalues = frame.closure.up_values.borrow();

                for upvalue_origin in func.up_values.iter() {
                    match *upvalue_origin {
                        // Create a new up-value pointing to a local variable
                        // in the current scope.
                        //
                        // Be mindful of terminology here.
                        // The current running closure is the *parent* of the child closure
                        // that is being spawned right now.
                        UpValueOrigin::Parent(local_id) => {
                            let stack_offset = frame.base + local_id as usize;
                            let up_value = Handle::new(UpValue::Open(stack_offset));
                            upvalues.push(up_value.clone());

                            // Keep a handle to the up-value in the current frame,
                            // so it can be closed when the local goes out of scope.
                            frame.up_values.push(up_value);
                        }
                        // Share a handle to an existing up-value.
                        UpValueOrigin::Outer(upvalue_id) => {
                            upvalues.push(parent_upvalues[upvalue_id as usize].clone());
                        }
                    }
                }

                let closure = Closure::with_up_values(func, upvalues.into_boxed_slice());
                let closure_rc = Rc::new(closure);
                vm.stack.push(Value::Object(Object::Closure(closure_rc)));
            }

            Op::Int_Neg => {
                let a = vm.stack[frame.ip].as_int().ok_or_else(err_int_expected)?;
                vm.stack[frame.ip] = Value::Int(-a);
            }
            Op::Int_Add => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::Int(a + b));
            }
            Op::Int_Sub => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::Int(a - b));
            }
            Op::Int_Mul => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::Int(a * b));
            }
            Op::Int_Div => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::Int(a / b));
            }
            Op::Int_Mod => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::Int(a % b));
            }

            Op::Int_Ne => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::from_bool(a != b));
            }
            Op::Int_Eq => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::from_bool(a == b));
            }
            Op::Int_Lt => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::from_bool(a < b));
            }
            Op::Int_Le => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::from_bool(a <= b));
            }
            Op::Int_Gt => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::from_bool(a > b));
            }
            Op::Int_Ge => {
                let [a, b] = vm.pop2_int()?;
                vm.stack.push(Value::from_bool(a >= b));
            }

            Op::Float_Neg => {
                let a = vm.stack[frame.ip].as_float().ok_or_else(err_float_expected)?;
                vm.stack[frame.ip] = Value::Float(-a);
            }
            Op::Float_Add => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::Float(a + b));
            }
            Op::Float_Sub => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::Float(a - b));
            }
            Op::Float_Mul => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::Float(a * b));
            }
            Op::Float_Div => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::Float(a / b));
            }
            Op::Float_Mod => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::Float(a % b));
            }

            Op::Float_Ne => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::from_bool(a != b));
            }
            Op::Float_Eq => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::from_bool(a == b));
            }
            Op::Float_Lt => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::from_bool(a < b));
            }
            Op::Float_Le => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::from_bool(a <= b));
            }
            Op::Float_Gt => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::from_bool(a > b));
            }
            Op::Float_Ge => {
                let [a, b] = vm.pop2_float()?;
                vm.stack.push(Value::from_bool(a >= b));
            }

            Op::Str_Concat => todo!(),
            Op::Str_Slice => todo!(),

            Op::JumpNe { addr } => {
                if vm.pop_int()? != 0 {
                    frame.jump(addr.into_i64())
                }
            }
            Op::JumpEq { addr } => {
                if vm.pop_int()? == 0 {
                    frame.jump(addr.into_i64())
                }
            }
            Op::JumpLt { addr } => {
                if vm.pop_int()? < 0 {
                    frame.jump(addr.into_i64())
                }
            }
            Op::JumpLe { addr } => {
                if vm.pop_int()? <= 0 {
                    frame.jump(addr.into_i64())
                }
            }
            Op::JumpGt { addr } => {
                let [a, b] = vm.pop2_int()?;
                if a > b {
                    frame.jump(addr.into_i64())
                }
            }
            Op::JumpGe { addr } => {
                if vm.pop_int()? >= 0 {
                    frame.jump(addr.into_i64())
                }
            }
            Op::JumpZero { addr } => {
                if vm.pop_int()? == 0 {
                    frame.jump(addr.into_i64())
                }
            }
            Op::Jump { addr } => frame.jump(addr.into_i64()),
        }
    }
}

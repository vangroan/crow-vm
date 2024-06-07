//! Objects (heap allocated reference types)
use std::fmt::{self, Formatter};
use std::rc::Rc;

use crate::handle::Handle;
use crate::op::Op;
use crate::value::Value;

#[derive(Clone)]
pub enum Object {
    Closure(Handle<Closure>),
    Func(Rc<Func>),
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Object::Closure(handle) => todo!(),
            Object::Func(rc) => write!(f, "Func(0x{:?})", Rc::as_ptr(rc)),
        }
    }
}

/// Function prototype.
pub struct Func {
    pub(crate) code: Box<[Op]>,

    /// The number of stack slots this function requires in its activation frame,
    /// including the callable object.
    pub(crate) stack_size: u32,

    /// Indicates whether the function takes variable arguments.
    pub(crate) is_varg: bool,

    pub(crate) constants: Constants,
}

pub struct Constants {
    pub(crate) ints: Box<[i64]>,
    pub(crate) floats: Box<[f64]>,
    pub(crate) strings: Box<[String]>,
    pub(crate) funcs: Box<[Rc<Func>]>,
}

struct FuncFmt<'a>(&'a Func);

impl<'a> fmt::Debug for FuncFmt<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Func({:?})", self.0 as *const _)
    }
}

/// A callable instance of a function, optionally with captured outer variables.
pub struct Closure {
    /// Shared handle to the function definition.
    ///
    /// Procedures are considered immutable after they're compiled,
    /// so we use [`Rc`] directly without the interior mutability
    /// offered by [`Handle`].
    pub(crate) proc: Rc<Func>,

    // TODO: Change to Box<[UpValue]>
    pub(crate) up_values: Vec<Handle<UpValue>>,
}

impl fmt::Debug for Closure {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let func_fmt = FuncFmt(&self.proc);
        f.debug_struct("Closure")
            .field("proc", &func_fmt)
            .field("up_values", &self.up_values)
            .finish()
    }
}

/// An Up-value is a variable that is referenced within a scope, but is not
/// local to that scope.
#[derive(Debug, Clone)]
pub enum UpValue {
    /// A local variable is an **open** up-value when it is still within scope
    /// and on the operand stack.
    ///
    /// In this case the up-value holds an absolute *stack offset* pointing to the
    /// local variable.
    Open(usize),

    /// A local variable is a **closed** up-value when the closure escapes its
    /// parent scope. The lifetime of those locals extend beyond their lexical
    /// scope, so they must be replaced with heap allocated values.
    ///
    /// In this case the up-value holds a *handle* to a heap value.
    Closed(Value),
}

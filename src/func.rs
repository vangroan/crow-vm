use std::rc::Rc;

use crate::op::Op;

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

use std::rc::Rc;

use crate::op::Op;

/// Function prototype.
pub struct Func {
    pub(crate) code: Box<[Op]>,

    /// Indicates whether the function takes variable arguments.
    pub(crate) is_varg: bool,
}

pub struct Constants {
    ints: Box<[i64]>,
    floats: Box<[f64]>,
    strings: Box<[String]>,
    funcs: Box<[Rc<Func>]>,
}

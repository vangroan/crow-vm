use std::rc::Rc;

use crate::errors::Result;
use crate::func::{Constants, Func};
use crate::op::{Arg24, Op};
use crate::vm::Vm;

#[test]
fn test_basic_math() -> Result<()> {
    let code = &[
        Op::PushInt(Arg24::from_i64(7)?),
        Op::PushInt(Arg24::from_i64(11)?),
        Op::Int_Add,
        Op::End,
    ];

    let func = Rc::new(Func {
        code: code.iter().cloned().collect(),
        stack_size: 3,
        is_varg: true,
        constants: Constants {
            ints: Box::new([]),
            floats: Box::new([]),
            strings: Box::new([]),
            funcs: Box::new([]),
        },
    });

    let env = ();

    let mut vm = Vm::new();

    vm.run(env, func)?;

    println!("stack: {:?}", vm.stack.last().unwrap().as_int());

    Ok(())
}

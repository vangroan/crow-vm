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

    println!("stack: {:?}", vm.stack);

    Ok(())
}

#[test]
fn test_basic_branch() -> Result<()> {
    let code = &[
        // locals a, b
        Op::PushInt(Arg24::from_i64(7)?),
        Op::PushInt(Arg24::from_i64(11)?),
        // if a > b
        Op::GetLocal { slot: 0 },
        Op::GetLocal { slot: 1 },
        Op::Int_Lt,
        Op::JumpZero {
            addr: Arg24::from_i64(2)?,
        },
        // then return 123
        Op::PushInt(Arg24::from_i64(123)?),
        Op::Return { results: 1 },
        // else
        Op::PushInt(Arg24::from_i64(456)?),
        Op::Return { results: 1 },
        Op::End,
    ];

    let func = Rc::new(Func {
        code: code.iter().cloned().collect(),
        stack_size: 4,
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

    println!("stack: {:?}", vm.stack);

    Ok(())
}

#[test]
fn test_basic_call() -> Result<()> {
    let add_func = Rc::new(Func {
        stack_size: 2,
        is_varg: false,
        constants: Constants {
            ints: Box::new([]),
            floats: Box::new([]),
            strings: Box::new([]),
            funcs: Box::new([]),
        },
        code: vec![Op::Int_Add, Op::Return { results: 1 }, Op::End].into_boxed_slice(),
    });

    let top_func = Rc::new(Func {
        stack_size: 6,
        is_varg: false,
        constants: Constants {
            ints: Box::new([]),
            floats: Box::new([]),
            strings: Box::new([]),
            funcs: Box::new([add_func]),
        },
        code: Box::new([
            // locals a, b
            Op::PushInt(Arg24::from_i64(7)?),
            Op::PushInt(Arg24::from_i64(11)?),
            Op::PushFunc(Arg24::from_u32(0)?),
            Op::GetLocal { slot: 1 },
            Op::GetLocal { slot: 2 },
            Op::Call {
                base: 3,
                results: 0,
            },
            Op::End,
        ]),
    });

    let mut vm = Vm::new();
    vm.run((), top_func)?;
    println!("stack: {:?}", vm.stack);

    Ok(())
}

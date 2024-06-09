use std::rc::Rc;

use crate::errors::Result;
use crate::object::{Constants, Func, UpValueOrigin};
use crate::op::{shorthand as op, Arg24, Op};
use crate::vm::Vm;

#[test]
fn test_basic_math() -> Result<()> {
    let code = &[
        Op::PushIntIn(Arg24::from_i64(7)?),
        Op::PushIntIn(Arg24::from_i64(11)?),
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
        up_values: Box::new([]),
    });

    let env = ();

    let mut vm = Vm::new();

    vm.run_function(env, func)?;

    println!("stack: {:?}", vm.stack);

    Ok(())
}

#[test]
fn test_basic_branch() -> Result<()> {
    let func = Rc::new(Func {
        stack_size: 4,
        is_varg: true,
        constants: Constants {
            ints: Box::new([]),
            floats: Box::new([]),
            strings: Box::new([]),
            funcs: Box::new([]),
        },
        up_values: Box::new([]),
        code: Box::new([
            // locals a, b
            Op::PushIntIn(Arg24::from_i64(7)?),
            Op::PushIntIn(Arg24::from_i64(11)?),
            // if a > b
            Op::GetLocal { slot: 1 },
            Op::GetLocal { slot: 2 },
            Op::Int_Lt,
            Op::JumpZero {
                addr: Arg24::from_i64(2)?,
            },
            // then return 123
            Op::PushIntIn(Arg24::from_i64(123)?),
            Op::Return { results: 1 },
            // else
            Op::PushIntIn(Arg24::from_i64(456)?),
            Op::Return { results: 1 },
            Op::End,
        ]),
    });

    let env = ();

    let mut vm = Vm::new();

    vm.run_function(env, func)?;

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
        up_values: Box::new([]),
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
        up_values: Box::new([]),
        code: Box::new([
            // locals a, b
            Op::PushIntIn(Arg24::from_i64(7)?),
            Op::PushIntIn(Arg24::from_i64(11)?),
            Op::PushFunc(Arg24::from_u32(0)?),
            Op::GetLocal { slot: 1 },
            Op::GetLocal { slot: 2 },
            Op::Call { base: 3, results: 1 },
            Op::End,
        ]),
    });

    let mut vm = Vm::new();
    vm.run_function((), top_func)?;
    println!("stack: {:?}", vm.stack);

    Ok(())
}

#[test]
fn test_recursion() -> Result<()> {
    const INPUT: i32 = 5;
    // local fib = func(n: Int) -> Int {
    //    if n <= 1 {
    //       return n
    //    }
    //    return fib(n-1) + fib(n-2)
    // };
    // TODO: Closures and up-values
    let fib_func = Rc::new(Func {
        stack_size: 7,
        is_varg: false,
        constants: Constants {
            ints: Box::new([]),
            floats: Box::new([]),
            strings: Box::new([]),
            funcs: Box::new([]),
        },
        up_values: Box::new([
            UpValueOrigin::Parent(1), // local fib = func...
        ]),
        code: vec![
            // .local 1, n:Int
            // if n >= 1 then
            op::get_local(1),
            op::push_int_inlined(1),
            op::jump_gt(1),
            op::return_(1), // return local 1
            // fib(n-2)
            op::get_upvalue(0),
            op::get_local(1),
            op::push_int_inlined(2),
            op::int_sub(),
            op::call(2, 1),
            // fib(n-1)
            op::get_upvalue(0),
            op::get_local(1),
            op::push_int_inlined(1),
            op::int_sub(),
            op::call(3, 1),
            // fib(n-1) + fib(n-2)
            op::int_add(),
            op::return_(1),
            op::end(),
        ]
        .into_boxed_slice(),
    });

    let top_func = Rc::new(Func {
        stack_size: 6,
        is_varg: false,
        constants: Constants {
            ints: Box::new([]),
            floats: Box::new([]),
            strings: Box::new([]),
            funcs: Box::new([fib_func]),
        },
        up_values: Box::new([]),
        code: Box::new([
            // local fib = func(n: Int) -> Int { ...
            op::create_closure(0),
            // fib(20)
            op::get_local(1),
            op::push_int_inlined(INPUT),
            op::call(2, 1),
            op::return_(1),
            op::end(),
        ]),
    });

    let mut vm = Vm::new();
    vm.run_function((), top_func)?;

    Ok(())
}

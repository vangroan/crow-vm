//! Crow scripting language.

mod array;
mod ast;
mod env;
mod errors;
mod handle;
mod lexer;
mod limits;
mod object;
mod op;
mod parser;
mod string;
#[cfg(test)]
mod tests;
mod token;
mod typechecker;
mod types;
mod value;
mod vm;

pub use op::{shorthand, Op};
pub use vm::Vm;

/// Compile the given source code text into an executable chunk.
pub fn compile(source: &str, filename: &str) -> self::errors::Result<()> {
    let lexer = self::lexer::Lexer::new(source, filename);
    let mut parser = self::parser::Parser::new(lexer);
    let block = parser.parse_module()?;
    println!("Syntax Tree:\n{block:#?}");
    let mut checker = self::typechecker::TypeChecker::new();
    let _ = checker.check_block(&block)?;

    // loop {
    //     let token = lexer.next_token()?;
    //     // println!("{token:?}");
    //     if matches!(token.kind, token::TokenKind::Eof) {
    //         break;
    //     }
    // }

    Ok(())
}

pub fn compile_file(filename: &str) -> self::errors::Result<()> {
    // TODO: Wrap std::io::Error
    let source_text = std::fs::read_to_string(filename).unwrap();
    compile(source_text.as_str(), filename)
}

/// Compile the given string as an expression.
///
/// Useful for REPL input.
pub fn compile_expr(_expression: &str) -> self::errors::Result<()> {
    todo!("compile bare expression")
}

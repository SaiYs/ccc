//! # implement notes
//!
//! Global scope can have only function definitions.
//! Entry point is function with a name `main`
//!
//! Function definition looks like `fn main() { \* stmt* *\ }` .
//! Only 64-bits signed integer is supported as Type.

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;

#[test]
fn test() {
    let s = include_str!("../input.sofa");
    let tokens = lexer::tokenize(s);
    dbg!(&tokens);

    let parser = parser::SofaParser::new(&tokens);
    let ast = parser.parse();
    dbg!(&ast);

    let mut generater = codegen::SofaGenerater::new(std::io::stdout());
    generater.gen(&ast);
}

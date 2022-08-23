use ccc::{codegen::Generater, lexer::tokenize, parser::Parser};

fn main() {
    let mut args = std::env::args();
    let source = args.nth(1).expect("error: invalid args");

    let tokens = tokenize(&source);

    let parser = Parser::new(tokens);
    let ast = parser.parse();

    let mut generater = Generater::new();
    generater.gen(&ast);
}

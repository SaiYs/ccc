pub mod codegen;
pub mod lexer;
pub mod parser;

#[test]
fn test() {
    let s = include_str!("../input.txt");
    let tokens = lexer::tokenize(s);
    dbg!(&tokens);

    let parser = parser::Parser::new(tokens);
    let ast = parser.parse();
    dbg!(&ast);

    let mut generater = codegen::Generater::new(std::io::stdout());
    generater.gen(&ast);
}

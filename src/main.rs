use ccc::{codegen::gen, parser::Node, tokenizer::Tokenizer};

fn main() {
    let mut args = std::env::args();
    let source = args.nth(1).expect("error: invalid args");
    let mut tokens = Tokenizer::new(source);
    let node = Node::new(&mut tokens);
    gen(node)
}

#[test]
fn test_tokenize() {
    let s = "a = 2; return a; return a;";
    let mut tokens = Tokenizer::new(s.to_string());
    dbg!(&tokens);

    let node = Node::new(&mut tokens);
    dbg!(&node);

    gen(node);
}

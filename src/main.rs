use ccc::{
    codegen::gen,
    parser::{Node, TokenList},
};

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        panic!("invalid args");
    }

    let source = args.nth(1).unwrap();
    let mut tokens = TokenList::new(source);
    let node = Node::new(&mut tokens);

    gen(node)
}

#[test]
fn test_tokenize() {
    // let s = "1 + 2 >= (4 - 1)";
    let s = "a = 4; a = a * 2; a";
    let mut tokens = TokenList::new(s.to_string());
    dbg!(&tokens);

    let node = Node::new(&mut tokens);
    dbg!(&node);

    gen(node);
}

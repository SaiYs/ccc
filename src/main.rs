use ccc::{parser::{TokenList, Node}, codegen::gen};

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        panic!("invalid args");
    }

    let source = args.nth(1).unwrap();
    let mut tokens = TokenList::new(source);
    let node = Node::new(&mut tokens);

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    gen(node);

    println!("    pop rax");
    println!("    ret");
}

#[test]
fn test_tokenize() {
    let s = "1 + 2 >= (4 - 1)";
    let mut tokens = TokenList::new(s.to_string());
    dbg!(&tokens);

    let node = Node::new(&mut tokens);
    dbg!(&node);
}

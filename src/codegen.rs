use crate::parser::{Node, NodeKind};

pub fn gen(node: Node) {
    if let NodeKind::Num(n) = node.kind {
        println!("    push {}", n);
    } else {
        gen(*node.lhs.unwrap());
        gen(*node.rhs.unwrap());

        println!("    pop rdi");
        println!("    pop rax");

        match node.kind {
            NodeKind::Add => println!("    add rax, rdi"),
            NodeKind::Sub => println!("    sub rax, rdi"),
            NodeKind::Mul => println!("    imul rax, rdi"),
            NodeKind::Div => {
                println!("    cqo");
                println!("    idiv rdi");
            }
            NodeKind::Num(_) => unreachable!(),
            NodeKind::Eq => {
                println!("    cmp rax, rdi");
                println!("    sete al");
                println!("    movzb rax, al");
            }
            NodeKind::Neq => {
                println!("    cmp rax, rdi");
                println!("    setne al");
                println!("    movzb rax, al");
            }
            NodeKind::Le => {
                println!("    cmp rax, rdi");
                println!("    setl al");
                println!("    movzb rax, al");
            }
            NodeKind::LeEq => {
                println!("    cmp rax, rdi");
                println!("    setle al");
                println!("    movzb rax, al");
            }
        }

        println!("    push rax");
    }
}

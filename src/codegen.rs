use crate::parser::{Local, Node, NodeKind};

pub fn gen(node: Node) {
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    println!("    push rbp");
    println!("    mov rbp, rsp");
    println!("    sub rsp, 208");

    gen_inner(node);

    println!("    mov rsp, rbp");
    println!("    pop rbp");
    println!("    ret");
}

fn gen_inner(node: Node) {
    match node.kind {
        NodeKind::Return => {
            gen_inner(*node.lhs.unwrap());

            println!("    pop rax");
            println!("    mov rsp, rbp");
            println!("    pop rbp");
            println!("    ret");
        }
        NodeKind::Semi => {
            if let Some(s) = node.lhs {
                gen_inner(*s);
            }
            println!("    pop rax");
            if let Some(s) = node.rhs {
                gen_inner(*s);
            }
        }
        NodeKind::Assign => {
            gen_lval(*node.lhs.unwrap());
            gen_inner(*node.rhs.unwrap());

            println!("    pop rdi");
            println!("    pop rax");
            println!("    mov [rax], rdi");
            println!("    push rdi");
        }
        NodeKind::Local(..) => {
            gen_lval(node);
            println!("    pop rax");
            println!("    mov rax, [rax]");
            println!("    push rax");
        }
        NodeKind::Num { value: n } => {
            println!("    push {}", n);
        }
        _ => {
            gen_inner(*node.lhs.unwrap());
            gen_inner(*node.rhs.unwrap());

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
                NodeKind::Num { .. } => unreachable!(),
                _ => {}
            }

            println!("    push rax");
        }
    }
}

fn gen_lval(node: Node) {
    if let Node {
        kind: NodeKind::Local(Local { offset, .. }),
        ..
    } = node
    {
        println!("    mov rax, rbp");
        println!("    sub rax, {}", offset);
        println!("    push rax");
    } else {
        panic!("left value of assignment statement must be local variable");
    }
}

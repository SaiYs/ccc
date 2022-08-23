use std::collections::HashMap;

use crate::parser::{Node, NodeKind};

#[derive(Debug, Default)]
pub struct Generater {
    local: HashMap<String, usize>,
}

impl Generater {
    pub fn new() -> Self {
        Self {
            local: HashMap::new(),
        }
    }

    pub fn gen(&mut self, node: &Node) {
        println!(".intel_syntax noprefix");
        println!(".global main");
        println!("main:");

        println!("    push rbp");
        println!("    mov rbp, rsp");
        println!("    sub rsp, 208");

        self.gen_inner(node);

        println!("    mov rsp, rbp");
        println!("    pop rbp");
        println!("    ret");
    }

    fn gen_inner(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::Return => {
                self.gen_inner(&node.children[0]);

                println!("    pop rax");
                println!("    mov rsp, rbp");
                println!("    pop rbp");
                println!("    ret");
            }
            NodeKind::Program => {
                for stmt in node.children.iter() {
                    self.gen_inner(stmt);
                    println!("    pop rax");
                }
            }
            NodeKind::Stmt => {
                self.gen_inner(&node.children[0]);
            }
            NodeKind::Assign => {
                self.gen_lval(&node.children[0]);
                self.gen_inner(&node.children[1]);

                println!("    pop rdi");
                println!("    pop rax");
                println!("    mov [rax], rdi");
                println!("    push rdi");
            }
            NodeKind::Ident(..) => {
                self.gen_lval(node);
                println!("    pop rax");
                println!("    mov rax, [rax]");
                println!("    push rax");
            }
            NodeKind::Num(num) => {
                println!("    push {}", num);
            }
            _ => {
                self.gen_inner(&node.children[0]);
                self.gen_inner(&node.children[1]);

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

    fn gen_lval(&mut self, node: &Node) {
        if let Node {
            kind: NodeKind::Ident(id),
            ..
        } = node
        {
            let l = self.local.len();
            let offset = self.local.entry(id.clone()).or_insert((l + 1) * 8);
            println!("    mov rax, rbp");
            println!("    sub rax, {}", offset);
            println!("    push rax");
        } else {
            panic!("left value of assignment statement must be local variable");
        }
    }
}

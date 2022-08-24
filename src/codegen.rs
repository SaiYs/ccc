use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};

use crate::parser::{BinOp, Node, NodeKind};

#[derive(Debug)]
pub struct Generater<W: Write> {
    local: HashMap<String, usize>,
    writer: BufWriter<W>,
    label_id: usize,
}

impl<W: Write> Generater<W> {
    pub fn new(writer: W) -> Self {
        Self {
            local: HashMap::new(),
            writer: BufWriter::new(writer),
            label_id: 0,
        }
    }

    pub fn gen(&mut self, node: &Node) {
        let entry_point = if cfg!(target_os = "macos") {
            "_main"
        } else {
            "main"
        };

        writeln!(self.writer, ".intel_syntax noprefix").unwrap();
        writeln!(self.writer, ".global {}", entry_point).unwrap();
        writeln!(self.writer, "{}:", entry_point).unwrap();

        writeln!(self.writer, "    push rbp").unwrap();
        writeln!(self.writer, "    mov rbp, rsp").unwrap();
        writeln!(self.writer, "    sub rsp, 256").unwrap(); // up to 32 local variables ( 256 = 2 ^ 8 )

        self.gen_inner(node);

        writeln!(self.writer, "    mov rsp, rbp").unwrap();
        writeln!(self.writer, "    pop rbp").unwrap();
        writeln!(self.writer, "    ret").unwrap();
    }

    fn gen_inner(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::Program => {
                for stmt in node.children.iter() {
                    self.gen_inner(stmt);
                    writeln!(self.writer, "    pop rax").unwrap();
                }
            }
            NodeKind::Stmt => {
                self.gen_inner(&node.children[0]);
            }
            NodeKind::Return => {
                self.gen_inner(&node.children[0]);

                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    mov rsp, rbp").unwrap();
                writeln!(self.writer, "    pop rbp").unwrap();
                writeln!(self.writer, "    ret").unwrap();
            }
            NodeKind::Block => {
                for stmt in node.children.iter() {
                    self.gen_inner(stmt);
                    writeln!(self.writer, "    pop rax").unwrap();
                }
                writeln!(self.writer, "    push rax").unwrap(); // last expr is return value of block
            }
            NodeKind::If => {
                if let Some(els) = node.children.get(2) {
                    // has else
                    let label_else = format!(".L{}", self.label_id);
                    self.label_id += 1;
                    let label_end = format!(".L{}", self.label_id);
                    self.label_id += 1;

                    self.gen_inner(&node.children[0]);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    cmp rax, 0").unwrap();
                    writeln!(self.writer, "    je {}", label_else).unwrap();
                    self.gen_inner(&node.children[1]);
                    writeln!(self.writer, "    jmp {}", label_end).unwrap();

                    writeln!(self.writer, "{}:", label_else).unwrap();
                    if els.kind == NodeKind::Block {
                        self.gen_inner(&node.children[2]);
                    } else {
                        panic!("else if is not yet implemented");
                    }
                    writeln!(self.writer, "{}:", label_end).unwrap();
                } else {
                    // no else
                    let label_end = format!(".L{}", self.label_id);
                    self.label_id += 1;

                    self.gen_inner(&node.children[0]);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    cmp rax, 0").unwrap();
                    writeln!(self.writer, "    je {}", label_end).unwrap();
                    self.gen_inner(&node.children[1]);
                    writeln!(self.writer, "{}:", label_end).unwrap();
                }
            }
            NodeKind::Assign => {
                self.gen_lval(&node.children[0]);
                self.gen_inner(&node.children[1]);

                writeln!(self.writer, "    pop rdi").unwrap();
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    mov [rax], rdi").unwrap();
                writeln!(self.writer, "    push rdi").unwrap();
            }
            NodeKind::Ident(..) => {
                self.gen_lval(node);
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    mov rax, [rax]").unwrap();
                writeln!(self.writer, "    push rax").unwrap();
            }
            NodeKind::Num(num) => {
                writeln!(self.writer, "    push {}", num).unwrap();
            }
            NodeKind::BinOp(binop) => {
                self.gen_inner(&node.children[0]);
                self.gen_inner(&node.children[1]);

                writeln!(self.writer, "    pop rdi").unwrap();
                writeln!(self.writer, "    pop rax").unwrap();

                match binop {
                    BinOp::Add => writeln!(self.writer, "    add rax, rdi").unwrap(),
                    BinOp::Sub => writeln!(self.writer, "    sub rax, rdi").unwrap(),
                    BinOp::Mul => writeln!(self.writer, "    imul rax, rdi").unwrap(),
                    BinOp::Div => {
                        writeln!(self.writer, "    cqo").unwrap();
                        writeln!(self.writer, "    idiv rdi").unwrap();
                    }
                    BinOp::Eq => {
                        writeln!(self.writer, "    cmp rax, rdi").unwrap();
                        writeln!(self.writer, "    sete al").unwrap();
                        writeln!(self.writer, "    movzb rax, al").unwrap();
                    }
                    BinOp::Neq => {
                        writeln!(self.writer, "    cmp rax, rdi").unwrap();
                        writeln!(self.writer, "    setne al").unwrap();
                        writeln!(self.writer, "    movzb rax, al").unwrap();
                    }
                    BinOp::Le => {
                        writeln!(self.writer, "    cmp rax, rdi").unwrap();
                        writeln!(self.writer, "    setl al").unwrap();
                        writeln!(self.writer, "    movzb rax, al").unwrap();
                    }
                    BinOp::LeEq => {
                        writeln!(self.writer, "    cmp rax, rdi").unwrap();
                        writeln!(self.writer, "    setle al").unwrap();
                        writeln!(self.writer, "    movzb rax, al").unwrap();
                    }
                }

                writeln!(self.writer, "    push rax").unwrap();
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
            writeln!(self.writer, "    mov rax, rbp").unwrap();
            writeln!(self.writer, "    sub rax, {}", offset).unwrap();
            writeln!(self.writer, "    push rax").unwrap();
        } else {
            panic!("left value of assignment statement must be local variable");
        }
    }
}

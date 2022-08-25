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

    fn gen_header(&mut self) {
        let entry_point = if cfg!(target_os = "macos") {
            "_main"
        } else {
            "main"
        };

        writeln!(self.writer, ".intel_syntax noprefix").unwrap();
        writeln!(self.writer, ".global {}", entry_point).unwrap();
        writeln!(self.writer, "{}:", entry_point).unwrap(); // entry point is main (_main)
    }

    pub fn gen(&mut self, node: &Node, stack_size: usize) {
        self.gen_header();

        // main
        self.gen_prologue();
        writeln!(self.writer, "    sub rsp, {}", 8 * stack_size).unwrap(); // epilogue

        self.gen_inner(node);

        self.gen_epilogue();
    }

    fn gen_prologue(&mut self) {
        writeln!(self.writer, "    push rbp").unwrap();
        writeln!(self.writer, "    mov rbp, rsp").unwrap();
    }

    fn gen_epilogue(&mut self) {
        writeln!(self.writer, "    leave").unwrap(); // equivelent to "mov rsp, rbp" and "pop rbp"
        writeln!(self.writer, "    ret").unwrap();
    }

    fn gen_inner(&mut self, node: &Node) {
        match &node.kind {
            NodeKind::Program => {
                for stmt in node.children.iter() {
                    self.gen_inner(stmt);
                }
            }
            NodeKind::Stmt => {
                self.gen_inner(&node.children[0]);
                writeln!(self.writer, "    pop rax").unwrap(); // discard remaining stack-top value
                writeln!(self.writer, "    push 0").unwrap(); // every stmt with semi left 0 on top of stack
            }
            NodeKind::Return => {
                self.gen_inner(&node.children[0]);

                writeln!(self.writer, "    pop rax").unwrap(); // return value is on top of stack
                writeln!(self.writer, "    leave").unwrap(); // equivelent to "mov rsp, rbp" and "pop rbp"
                writeln!(self.writer, "    ret").unwrap();
            }
            NodeKind::Block => {
                for stmt in node.children.iter() {
                    self.gen_inner(stmt);
                }
                // writeln!(self.writer, "    push rax").unwrap(); // last expr is return value of block, but disabled for simplicity
            }
            NodeKind::Loop => {
                let label = format!(".L{}_loop", self.label_id);
                self.label_id += 1;

                writeln!(self.writer, "{}:", label).unwrap();
                self.gen_inner(&node.children[0]);
                writeln!(self.writer, "    jmp {}", label).unwrap();
            }
            NodeKind::If => {
                if let Some(els) = node.children.get(2) {
                    // has else
                    let label_else = format!(".L{}_else", self.label_id);
                    self.label_id += 1;
                    let label_end = format!(".L{}_end", self.label_id);
                    self.label_id += 1;

                    self.gen_inner(&node.children[0]);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    cmp rax, 0").unwrap();
                    writeln!(self.writer, "    je {}", label_else).unwrap();
                    self.gen_inner(&node.children[1]); // if block
                    writeln!(self.writer, "    jmp {}", label_end).unwrap();

                    writeln!(self.writer, "{}:", label_else).unwrap();
                    if els.kind == NodeKind::Block {
                        self.gen_inner(&node.children[2]); // else block
                    } else {
                        panic!("\"else if\" is not yet implemented"); // else if block
                    }
                    writeln!(self.writer, "{}:", label_end).unwrap();
                } else {
                    // no else
                    let label_end = format!(".L{}_end", self.label_id);
                    self.label_id += 1;

                    self.gen_inner(&node.children[0]);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    cmp rax, 0").unwrap();
                    writeln!(self.writer, "    je {}", label_end).unwrap();
                    self.gen_inner(&node.children[1]); // if block
                    writeln!(self.writer, "{}:", label_end).unwrap();
                }
            }
            NodeKind::Assign => {
                self.gen_lval(&node.children[0]); // lval shoud be address-able
                self.gen_inner(&node.children[1]);

                writeln!(self.writer, "    pop rdi").unwrap(); // rhs value
                writeln!(self.writer, "    pop rax").unwrap(); // lhs local's address
                writeln!(self.writer, "    mov [rax], rdi").unwrap();
                // writeln!(self.writer, "    push rdi").unwrap(); // nesting assign stmt is disabled for simplicity
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
            NodeKind::FuncCall(..) => {
                todo!()
            }
            NodeKind::Ident(..) => {
                self.gen_lval(node);
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    mov rax, [rax]").unwrap(); // address into value on itself
                writeln!(self.writer, "    push rax").unwrap();
            }
            NodeKind::Num(num) => {
                writeln!(self.writer, "    push {}", num).unwrap(); // num is imm
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
            writeln!(self.writer, "    mov rax, rbp").unwrap(); // retrieve rbp into rax
            writeln!(self.writer, "    sub rax, {}", offset).unwrap(); // local stored at offset from rbp
            writeln!(self.writer, "    push rax").unwrap(); // return local's address
        } else {
            panic!("left value of assignment statement must be local variable");
        }
    }
}

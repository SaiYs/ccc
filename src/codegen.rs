use std::{
    collections::HashMap,
    io::{BufWriter, Write},
};

use crate::{
    ast::{
        Assign, Ast, BinOp, BinOpKind, Block, Enclosed, Expr, FnCall, FnDef, Global, IfElse, Init,
        Loop, Number, Return, Stmt, UnOp, UnOpKind,
    },
    ty::Type,
};

const MAX_STACK_SIZE: usize = 8 * 256;
const ARG_REGS: [&str; 6] = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];

#[derive(Debug)]
pub struct SofaGenerater<W: Write> {
    writer: BufWriter<W>,
    label_id: usize,
    offset: usize,
    offset_table: HashMap<String, usize>,
}

impl<W: Write> SofaGenerater<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer: BufWriter::new(writer),
            label_id: 0,
            offset: 0,
            offset_table: HashMap::new(),
        }
    }

    fn gen_header(&mut self) {
        let entry_point = "main";

        writeln!(self.writer, ".intel_syntax noprefix").unwrap();
        writeln!(self.writer, ".global {}", entry_point).unwrap();
        writeln!(self.writer).unwrap();
    }

    pub fn gen(&mut self, ast: &Ast) {
        self.gen_header();

        self.gen_global(&ast.node);

        writeln!(self.writer).unwrap();
    }

    fn gen_global(&mut self, global: &Global) {
        for f in global.definitions.iter() {
            self.gen_fn(f);
            self.offset = 0;
            self.offset_table.clear();
        }
    }

    fn gen_fn(&mut self, f: &FnDef) {
        // stack_size should be a multiple of 16;
        let (name, stack_size) = if f.name == "main" {
            ("main".to_string(), MAX_STACK_SIZE)
        } else {
            (f.name.clone(), (f.args.len() + 1) / 2 * 2 * 8)
        };

        self.gen_prologue(&name, stack_size);

        if !f.args.is_empty() {
            writeln!(self.writer, "    mov rax, rbp").unwrap();
        }
        for (arg, reg) in f.args.iter().zip(ARG_REGS) {
            let size = arg.ty.size();
            self.offset += size;
            self.offset_table.insert(arg.name.clone(), self.offset);

            writeln!(self.writer, "    sub rax, {}", size).unwrap();
            writeln!(self.writer, "    mov [rax], {}", reg).unwrap();
        }

        self.gen_block(&f.body);
        self.gen_epilogue();
    }

    fn gen_prologue(&mut self, name: &str, stack_size: usize) {
        writeln!(self.writer, "{}:", name).unwrap();
        writeln!(self.writer, "    push rbp").unwrap();
        writeln!(self.writer, "    mov rbp, rsp").unwrap();
        writeln!(self.writer, "    sub rsp, {}", stack_size).unwrap();
    }

    fn gen_epilogue(&mut self) {
        writeln!(self.writer, "    leave").unwrap(); // equivalent to "mov rsp, rbp" and "pop rbp"
        writeln!(self.writer, "    ret").unwrap();
    }

    fn gen_block(&mut self, block: &Block) {
        for expr in block.exprs.iter() {
            self.gen_expr(expr);
        }
    }

    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Stmt(Stmt { expr }) => {
                self.gen_expr(expr);
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    push 0").unwrap(); // unit
                writeln!(self.writer).unwrap();
            }
            Expr::Block(block) => self.gen_block(block),
            Expr::Return(Return { expr }) => {
                self.gen_expr(expr);
                writeln!(self.writer, "    pop rax").unwrap();
                self.gen_epilogue();
            }
            Expr::Loop(Loop { body }) => {
                let label = format!(".L{}_loop", self.label_id);
                self.label_id += 1;

                writeln!(self.writer, "{}:", label).unwrap();

                self.gen_block(body);

                writeln!(self.writer, "    jmp {}", label).unwrap();
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    push 1").unwrap(); // never
            }
            Expr::IfElse(IfElse {
                cond,
                if_body,
                else_body,
            }) => {
                if let Some(else_body) = else_body {
                    let label_else = format!(".L{}_else", self.label_id);
                    self.label_id += 1;
                    let label_end = format!(".L{}_end", self.label_id);
                    self.label_id += 1;

                    assert_eq!(cond.ty(), Type::Bool);
                    self.gen_expr(cond);

                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    cmp rax, 0").unwrap();
                    writeln!(self.writer, "    je {}", label_else).unwrap();
                    self.gen_block(if_body);
                    writeln!(self.writer, "    jmp {}", label_end).unwrap();

                    writeln!(self.writer, "{}:", label_else).unwrap();
                    self.gen_block(else_body);

                    writeln!(self.writer, "{}:", label_end).unwrap();
                } else {
                    let label_end = format!(".L{}_end", self.label_id);
                    self.label_id += 1;

                    self.gen_expr(cond);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    cmp rax, 0").unwrap();
                    writeln!(self.writer, "    je {}", label_end).unwrap();
                    self.gen_block(if_body);
                    writeln!(self.writer, "{}:", label_end).unwrap();
                }
            }
            Expr::FnCall(FnCall { name, args, .. }) => {
                for (expr, reg) in args.iter().zip(ARG_REGS) {
                    self.gen_expr(expr);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    mov {}, rax", reg).unwrap();
                }
                writeln!(self.writer, "    call {}", name).unwrap();
                writeln!(self.writer, "    push rax").unwrap();
            }
            Expr::Init(Init { name, value }) => {
                if let Expr::Local(local) = &**name {
                    let size = local.ty.size();
                    self.offset += size;
                    self.offset_table.insert(local.name.clone(), self.offset);

                    writeln!(self.writer, "    mov rax, rbp").unwrap(); // retrieve rbp into rax
                    writeln!(self.writer, "    sub rax, {}", self.offset).unwrap(); // local stored at offset from rbp
                    writeln!(self.writer, "    push rax").unwrap(); // return local's address

                    if let Some(value) = value {
                        assert_eq!(local.ty, value.ty());

                        self.gen_expr(value);

                        writeln!(self.writer, "    pop rdi").unwrap();
                        writeln!(self.writer, "    pop rax").unwrap();
                        writeln!(self.writer, "    mov [rax], rdi").unwrap();
                        writeln!(self.writer, "    push 0").unwrap(); // void
                    } else {
                        writeln!(self.writer, "    pop rax").unwrap();
                        writeln!(self.writer, "    push 0").unwrap(); // void
                    }
                } else {
                    panic!("lhs must be addressable")
                }
            }
            Expr::Assign(Assign { lhs, rhs }) => {
                match &**lhs {
                    Expr::UnOp(UnOp {
                        kind: UnOpKind::Deref,
                        expr,
                    }) => {
                        self.gen_expr(expr);
                    }
                    _ => {
                        assert_eq!(lhs.ty(), rhs.ty());
                        self.gen_address(lhs);
                    }
                }
                self.gen_expr(rhs);

                writeln!(self.writer, "    pop rdi").unwrap();
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    mov [rax], rdi").unwrap();
                writeln!(self.writer, "    push 0").unwrap(); // void
            }
            Expr::BinOp(binop) => {
                self.gen_binop(binop);
            }
            Expr::UnOp(UnOp { kind, expr }) => match kind {
                UnOpKind::Neg => {
                    self.gen_expr(expr);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    neg rax").unwrap();
                    writeln!(self.writer, "    push rax").unwrap();
                }
                UnOpKind::Ref => {
                    // TODO:
                    // take reference of imm is not yet supported,
                    // like, &10 or &&a
                    self.gen_address(expr);
                }
                UnOpKind::Deref => {
                    self.gen_expr(expr);
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    mov rax, [rax]").unwrap();
                    writeln!(self.writer, "    push rax").unwrap();
                }
            },
            Expr::Enclosed(Enclosed { expr }) => self.gen_expr(expr),
            Expr::Bool(boolean) => match boolean {
                crate::ast::Bool::True => writeln!(self.writer, "    push 1").unwrap(),
                crate::ast::Bool::False => writeln!(self.writer, "    push 0").unwrap(),
            },
            Expr::Local(local) => {
                self.gen_address(expr);

                if matches!(local.ty, Type::Array { .. }) {
                    // leave address
                } else {
                    writeln!(self.writer, "    pop rax").unwrap();
                    writeln!(self.writer, "    mov rax, [rax]").unwrap(); // address into value on itself
                    writeln!(self.writer, "    push rax").unwrap();
                }
            }
            Expr::Number(Number { value }) => writeln!(self.writer, "    push {}", value).unwrap(), // num is imm
        }
    }

    fn gen_address(&mut self, expr: &Expr) {
        match expr {
            Expr::Local(local) => {
                let offset = self
                    .offset_table
                    .get(&local.name)
                    .expect("found undefined local");

                writeln!(self.writer, "    mov rax, rbp").unwrap(); // retrieve rbp into rax
                writeln!(self.writer, "    sub rax, {}", offset).unwrap(); // local stored at offset from rbp
                writeln!(self.writer, "    push rax").unwrap(); // return local's address
            }
            Expr::UnOp(UnOp {
                kind: UnOpKind::Deref,
                expr,
            }) => match &**expr {
                Expr::UnOp(_) => self.gen_address(expr),
                Expr::Local(_) => {
                    self.gen_expr(expr);
                }
                _ => panic!("invalid lval {:?}", expr),
            },
            _ => panic!("invalid lval {:?}", expr),
        }
    }

    fn gen_binop(&mut self, BinOp { op, lhs, rhs }: &BinOp) {
        match (op, lhs.ty(), rhs.ty()) {
            (
                BinOpKind::Add
                | BinOpKind::Sub
                | BinOpKind::Mul
                | BinOpKind::Div
                | BinOpKind::Rem
                | BinOpKind::BitAnd
                | BinOpKind::BitOr
                | BinOpKind::BitXor,
                Type::I64,
                Type::I64,
            ) => self.gen_math(op, lhs, rhs),

            (
                BinOpKind::Eq
                | BinOpKind::Neq
                | BinOpKind::LeEq
                | BinOpKind::Le
                | BinOpKind::GtEq
                | BinOpKind::Gt,
                Type::I64,
                Type::I64,
            ) => self.gen_cmp(op, lhs, rhs),

            (BinOpKind::Add, Type::Ptr { to }, Type::I64) => {
                self.gen_expr(lhs);
                self.gen_expr(rhs);

                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    imul rax, {}", to.size()).unwrap();
                writeln!(self.writer, "    push rax").unwrap();
                writeln!(self.writer, "    pop rdi").unwrap();
                writeln!(self.writer, "    pop rax").unwrap();

                writeln!(self.writer, "    add rax, rdi").unwrap();
            }

            (BinOpKind::Sub, Type::Ptr { to }, Type::I64) => {
                self.gen_expr(lhs);
                self.gen_expr(rhs);

                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    imul rax, {}", to.size()).unwrap();
                writeln!(self.writer, "    push rax").unwrap();
                writeln!(self.writer, "    pop rdi").unwrap();
                writeln!(self.writer, "    pop rax").unwrap();

                writeln!(self.writer, "    sub rax, rdi").unwrap();
            }

            (BinOpKind::Add, Type::Array { element, len: _ }, Type::I64) => {
                self.gen_expr(lhs);
                self.gen_expr(rhs);

                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    imul rax, {}", element.size()).unwrap();
                writeln!(self.writer, "    push rax").unwrap();
                writeln!(self.writer, "    pop rdi").unwrap();
                writeln!(self.writer, "    pop rax").unwrap();

                writeln!(self.writer, "    add rax, rdi").unwrap();
            }

            (BinOpKind::LogAnd, Type::Bool, Type::Bool) => {
                let label1 = format!(".L{}_short", self.label_id);
                self.label_id += 1;
                let label2 = format!(".L{}_short", self.label_id);
                self.label_id += 1;

                self.gen_expr(lhs);
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    cmp rax, 0").unwrap();
                writeln!(self.writer, "    je {}", label1).unwrap();

                self.gen_expr(rhs);
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    cmp rax, 0").unwrap();
                writeln!(self.writer, "    je {}", label1).unwrap();

                writeln!(self.writer, "    mov rax, 1").unwrap();
                writeln!(self.writer, "    jmp {}", label2).unwrap();

                writeln!(self.writer, "{}:", label1).unwrap();
                writeln!(self.writer, "    mov rax, 0").unwrap();

                writeln!(self.writer, "{}:", label2).unwrap();
            }

            (BinOpKind::LogOr, Type::Bool, Type::Bool) => {
                let label1 = format!(".L{}_short", self.label_id);
                self.label_id += 1;
                let label2 = format!(".L{}_short", self.label_id);
                self.label_id += 1;
                let label3 = format!(".L{}_short", self.label_id);
                self.label_id += 1;

                self.gen_expr(lhs);
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    cmp rax, 0").unwrap();
                writeln!(self.writer, "    jne {}", label1).unwrap();

                self.gen_expr(rhs);
                writeln!(self.writer, "    pop rax").unwrap();
                writeln!(self.writer, "    cmp rax, 0").unwrap();
                writeln!(self.writer, "    je {}", label2).unwrap();

                writeln!(self.writer, "{}:", label1).unwrap();
                writeln!(self.writer, "    mov rax, 1").unwrap();
                writeln!(self.writer, "    jmp {}", label3).unwrap();

                writeln!(self.writer, "{}:", label2).unwrap();
                writeln!(self.writer, "    mov rax, 0").unwrap();

                writeln!(self.writer, "{}:", label3).unwrap();
            }

            _ => panic!("{:?} for {:?} and {:?} is not implemented", op, lhs, rhs),
        }

        writeln!(self.writer, "    push rax").unwrap();
    }

    fn gen_math(&mut self, op: &BinOpKind, lhs: &Expr, rhs: &Expr) {
        self.gen_expr(lhs);
        self.gen_expr(rhs);

        writeln!(self.writer, "    pop rdi").unwrap();
        writeln!(self.writer, "    pop rax").unwrap();

        match op {
            BinOpKind::Add => writeln!(self.writer, "    add rax, rdi").unwrap(),
            BinOpKind::Sub => writeln!(self.writer, "    sub rax, rdi").unwrap(),
            BinOpKind::Mul => writeln!(self.writer, "    imul rax, rdi").unwrap(),
            BinOpKind::Div => {
                writeln!(self.writer, "    cqo").unwrap();
                writeln!(self.writer, "    idiv rdi").unwrap();
            }
            BinOpKind::Rem => {
                writeln!(self.writer, "    cqo").unwrap();
                writeln!(self.writer, "    idiv rdi").unwrap();
                writeln!(self.writer, "    mov rax, rdx").unwrap();
            }
            BinOpKind::BitAnd => writeln!(self.writer, "    and rax, rdi").unwrap(),
            BinOpKind::BitOr => writeln!(self.writer, "    or rax, rdi").unwrap(),
            BinOpKind::BitXor => writeln!(self.writer, "    xor rax, rdi").unwrap(),
            _ => unreachable!(),
        };
    }

    fn gen_cmp(&mut self, op: &BinOpKind, lhs: &Expr, rhs: &Expr) {
        self.gen_expr(lhs);
        self.gen_expr(rhs);
        if matches!(op, BinOpKind::Gt | BinOpKind::GtEq) {
            writeln!(self.writer, "    pop rax").unwrap();
            writeln!(self.writer, "    pop rdi").unwrap();
        } else {
            writeln!(self.writer, "    pop rdi").unwrap();
            writeln!(self.writer, "    pop rax").unwrap();
        }
        writeln!(self.writer, "    cmp rax, rdi").unwrap();
        writeln!(
            self.writer,
            "    {} al",
            match op {
                BinOpKind::Eq => "sete",
                BinOpKind::Neq => "setne",
                BinOpKind::LeEq | BinOpKind::GtEq => "setle",
                BinOpKind::Le | BinOpKind::Gt => "setl",
                _ => unreachable!(),
            }
        )
        .unwrap();
        writeln!(self.writer, "    movzb rax, al").unwrap();
    }
}

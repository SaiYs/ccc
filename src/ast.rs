use crate::ty::Type;

#[derive(Debug)]
pub struct Ast {
    pub node: Global,
}

#[derive(Debug)]
pub struct Global {
    pub definitions: Vec<FnDef>,
}

#[derive(Debug)]
pub struct FnDef {
    pub name: String,
    pub args: Vec<Local>,
    pub fn_type: Type,
    pub body: Block,
}

#[derive(Debug)]
pub enum Expr {
    Stmt(Stmt),
    Block(Block),
    Return(Return),
    Loop(Loop),
    IfElse(IfElse),
    FnCall(FnCall),
    Init(Init),
    Assign(Assign),
    BinOp(BinOp),
    UnOp(UnOp),
    Enclosed(Enclosed),
    Local(Local),
    Number(Number),
}

impl Expr {
    pub fn ty(&self) -> Type {
        match self {
            Expr::Stmt(_) => Type::Void,
            Expr::Block(Block { exprs }) => {
                exprs.last().map_or(Type::Void, |last_expr| last_expr.ty())
            }
            Expr::Return(_) => Type::Never,
            Expr::Loop(_) => Type::Never,
            Expr::IfElse(IfElse {
                cond: _,
                if_body,
                else_body: _,
            }) => if_body
                .exprs
                .last()
                .map_or(Type::Void, |last_expr| last_expr.ty()),
            Expr::FnCall(FnCall { fn_type, .. }) => {
                if let Type::Fn { ret, .. } = fn_type {
                    *ret.clone()
                } else {
                    panic!("function's type must be Fn")
                }
            }
            Expr::Init(_) => Type::Void,
            Expr::Assign(_) => Type::Void,
            Expr::BinOp(BinOp { op, lhs, rhs }) => match (op, lhs.ty(), rhs.ty()) {
                (_, Type::I64, Type::I64) => Type::I64,
                (BinOpKind::Add | BinOpKind::Sub, Type::Ptr { to }, Type::I64) => Type::Ptr { to },
                (BinOpKind::Add, Type::Array { element, .. }, Type::I64) => {
                    Type::Ptr { to: element }
                }
                _ => panic!("{:?} is not defined between {:?} and {:?}", op, lhs, rhs),
            },
            Expr::UnOp(UnOp { kind, expr }) => match kind {
                UnOpKind::Neg => expr.ty(),
                UnOpKind::Ref => Type::Ptr {
                    to: Box::new(expr.ty()),
                },
                UnOpKind::Deref => match expr.ty() {
                    Type::Ptr { to } => *to,
                    Type::Array { element, .. } => *element,
                    _ => panic!("only pointer type can be dereferenced"),
                },
            },
            Expr::Enclosed(Enclosed { expr }) => expr.ty(),
            Expr::Local(Local { ty, .. }) => ty.clone(),
            Expr::Number(_) => Type::I64,
        }
    }
}

#[derive(Debug)]
pub struct Stmt {
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct Block {
    pub exprs: Vec<Expr>,
}

#[derive(Debug)]
pub struct Return {
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct Loop {
    pub body: Block,
}

#[derive(Debug)]
pub struct IfElse {
    pub cond: Box<Expr>,
    pub if_body: Block,
    pub else_body: Option<Block>,
}

#[derive(Debug)]
pub struct FnCall {
    pub name: String,
    pub args: Vec<Expr>,
    pub fn_type: Type,
}

#[derive(Debug)]
pub struct Init {
    pub name: Box<Expr>,
    pub value: Option<Box<Expr>>,
}

#[derive(Debug)]
pub struct Assign {
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug)]
pub struct BinOp {
    pub op: BinOpKind,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOpKind {
    // low priority
    Eq,
    Neq,

    LeEq,
    Le,
    GeEq,
    Ge,

    Add,
    Sub,

    Mul,
    Div,
    // high priority
}

#[derive(Debug)]
pub struct UnOp {
    pub kind: UnOpKind,
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub enum UnOpKind {
    Neg,
    Ref,
    Deref,
}

#[derive(Debug)]

pub struct Enclosed {
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct Local {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug)]
pub struct Number {
    pub value: String,
}

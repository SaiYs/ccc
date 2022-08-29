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
    pub args: Vec<(Local, Type)>,
    pub return_type: Type,
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
}

#[derive(Debug)]
pub struct Init {
    pub name: Box<Expr>,
    pub ty: Type,
    pub value: Box<Expr>,
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
pub enum Type {
    I64,
    Ptr(Box<Type>),
    Void,
}

#[derive(Debug)]

pub struct Enclosed {
    pub expr: Box<Expr>,
}

#[derive(Debug)]
pub struct Local {
    pub name: String,
}

#[derive(Debug)]
pub struct Number {
    pub value: String,
}

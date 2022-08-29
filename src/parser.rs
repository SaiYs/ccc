use crate::{
    ast::{
        Assign, Ast, BinOp, BinOpKind, Block, Enclosed, Expr, FnCall, FnDef, Global, IfElse, Init,
        Local, Loop, Number, Return, Stmt, Type, UnOp, UnOpKind,
    },
    lexer::{Token, TokenKind},
};

pub struct SofaParser<'ctx> {
    head: usize,
    tokens: &'ctx [Token],
}

impl<'ctx> SofaParser<'ctx> {
    pub fn new(tokens: &'ctx [Token]) -> Self {
        Self { head: 0, tokens }
    }

    fn is_eof(&mut self) -> bool {
        self.head >= self.tokens.len()
    }

    fn get(&self) -> &Token {
        &self.tokens[self.head]
    }

    fn skip(&mut self) {
        self.head += 1;
    }

    fn peek(&mut self, target: &[TokenKind]) -> bool {
        (0..target.len()).all(|i| !self.is_eof() && self.tokens[self.head + i].kind == target[i])
    }

    fn consume(&mut self, target: &[TokenKind]) -> bool {
        if self.peek(target) {
            self.head += target.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, target: &[TokenKind]) {
        if !(self.consume(target)) {
            panic!("found {:?}, not {:?}", self.tokens[self.head], target)
        }
    }

    fn consume_operator(&mut self) -> Option<BinOpKind> {
        if self.consume(&[TokenKind::Eq, TokenKind::Eq]) {
            Some(BinOpKind::Eq)
        } else if self.consume(&[TokenKind::Bang, TokenKind::Eq]) {
            Some(BinOpKind::Neq)
        } else if self.consume(&[TokenKind::Lt, TokenKind::Eq]) {
            Some(BinOpKind::LeEq)
        } else if self.consume(&[TokenKind::Lt]) {
            Some(BinOpKind::Le)
        } else if self.consume(&[TokenKind::Gt, TokenKind::Eq]) {
            Some(BinOpKind::GeEq)
        } else if self.consume(&[TokenKind::Gt]) {
            Some(BinOpKind::Ge)
        } else if self.consume(&[TokenKind::Plus]) {
            Some(BinOpKind::Add)
        } else if self.consume(&[TokenKind::Minus]) {
            Some(BinOpKind::Sub)
        } else if self.consume(&[TokenKind::Star]) {
            Some(BinOpKind::Mul)
        } else if self.consume(&[TokenKind::Slash]) {
            Some(BinOpKind::Div)
        } else {
            None
        }
    }

    fn expect_ident(&mut self) -> String {
        let id = self.tokens[self.head].value.clone();
        self.expect(&[TokenKind::Ident]);
        id.unwrap()
    }

    fn expect_number(&mut self) -> String {
        let id = self.tokens[self.head].value.clone();
        self.expect(&[TokenKind::Number]);
        id.unwrap()
    }
}

impl<'ctx> SofaParser<'ctx> {
    pub fn parse(mut self) -> Ast {
        Ast {
            node: self.global(),
        }
    }

    fn global(&mut self) -> Global {
        let mut res = Global {
            definitions: vec![],
        };

        loop {
            if self.is_eof() {
                break res;
            } else {
                res.definitions.push(self.fn_def());
            }
        }
    }

    fn fn_def(&mut self) -> FnDef {
        self.expect(&[TokenKind::Fn]);
        let id = self.expect_ident();

        self.expect(&[TokenKind::LParen]);
        let mut args = vec![];
        while !self.consume(&[TokenKind::RParen]) {
            let local = self.local();
            self.expect(&[TokenKind::Colon]);
            let ty = self.ty();
            args.push((local, ty));
            self.consume(&[TokenKind::Comma]);
        }

        let return_type = if self.consume(&[TokenKind::Minus, TokenKind::Gt]) {
            self.ty()
        } else {
            Type::Void
        };

        FnDef {
            name: id,
            args,
            return_type,
            body: self.block(),
        }
    }

    fn block(&mut self) -> Block {
        self.expect(&[TokenKind::LBrace]);

        let mut res = Block { exprs: vec![] };
        while !self.consume(&[TokenKind::RBrace]) {
            let expr = self.expr();
            res.exprs.push(if self.consume(&[TokenKind::Semi]) {
                Expr::Stmt(Stmt {
                    expr: Box::new(expr),
                })
            } else {
                expr
            });
        }
        res
    }

    fn expr(&mut self) -> Expr {
        // eager
        let res = if self.peek(&[TokenKind::LBrace]) {
            Expr::Block(self.block())
        } else if self.consume(&[TokenKind::Return]) {
            Expr::Return(Return {
                expr: Box::new(self.expr()),
            })
        } else if self.consume(&[TokenKind::Loop]) {
            Expr::Loop(Loop { body: self.block() })
        } else if self.peek(&[TokenKind::If]) {
            Expr::IfElse(self.ifelse())
        } else if self.peek(&[TokenKind::Ident, TokenKind::LParen]) {
            Expr::FnCall(self.fn_call())
        } else if self.peek(&[TokenKind::Let]) {
            return Expr::Init(self.init());
        } else if self.consume(&[TokenKind::And]) {
            Expr::UnOp(UnOp {
                kind: UnOpKind::Ref,
                expr: Box::new(self.expr()),
            })
        } else if self.consume(&[TokenKind::Star]) {
            Expr::UnOp(UnOp {
                kind: UnOpKind::Deref,
                expr: Box::new(Expr::Local(Local {
                    name: self.expect_ident(),
                })),
            })
        } else if self.consume(&[TokenKind::Minus]) {
            Expr::UnOp(UnOp {
                kind: UnOpKind::Neg,
                expr: Box::new(self.expr()),
            })
        } else if self.consume(&[TokenKind::LParen]) {
            let res = Expr::Enclosed(Enclosed {
                expr: Box::new(self.expr()),
            });
            self.expect(&[TokenKind::RParen]);
            res
        } else if self.peek(&[TokenKind::Ident]) {
            Expr::Local(self.local())
        } else if self.peek(&[TokenKind::Number]) {
            Expr::Number(self.number())
        } else {
            panic!("found {:?}", self.get())
        };

        if self.consume(&[TokenKind::Eq]) {
            Expr::Assign(Assign {
                lhs: Box::new(res),
                rhs: Box::new(self.expr()),
            })
        } else if let Some(op) = self.consume_operator() {
            Expr::BinOp(BinOp {
                op,
                lhs: Box::new(res),
                rhs: Box::new(self.expr()),
            })
        } else {
            res
        }
    }

    fn ifelse(&mut self) -> IfElse {
        self.expect(&[TokenKind::If]);
        IfElse {
            cond: Box::new(self.expr()),
            if_body: self.block(),
            else_body: self.consume(&[TokenKind::Else]).then(|| self.block()),
        }
    }

    fn fn_call(&mut self) -> FnCall {
        let name = self.expect_ident();
        self.skip();

        let mut args = vec![];
        while !self.consume(&[TokenKind::RParen]) {
            args.push(self.expr());
            self.consume(&[TokenKind::Comma]);
        }
        FnCall { name, args }
    }

    fn init(&mut self) -> Init {
        self.expect(&[TokenKind::Let]);
        let name = Box::new(Expr::Local(self.local()));
        self.expect(&[TokenKind::Colon]);
        let ty = self.ty();
        self.expect(&[TokenKind::Eq]);
        let value = Box::new(self.expr());
        Init { name, ty, value }
    }

    fn ty(&mut self) -> Type {
        if self.consume(&[TokenKind::And]) {
            Type::Ptr(Box::new(self.ty()))
        } else {
            let id = self.expect_ident();
            match id.as_str() {
                "i64" => Type::I64,
                _ => panic!("found unknown type {}", id),
            }
        }
    }

    fn local(&mut self) -> Local {
        Local {
            name: self.expect_ident(),
        }
    }

    fn number(&mut self) -> Number {
        Number {
            value: self.expect_number(),
        }
    }
}

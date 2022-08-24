use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    Program,

    Assign,
    Stmt,
    Block,
    If,
    Return,

    BinOp(BinOp),

    Num(String),
    Ident(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Le,
    LeEq,
}

impl ToString for BinOp {
    fn to_string(&self) -> String {
        match self {
            BinOp::Add => "ADD",
            BinOp::Sub => "SUB",
            BinOp::Mul => "MUL",
            BinOp::Div => "DIV",
            BinOp::Eq => "EQ",
            BinOp::Neq => "NEQ",
            BinOp::Le => "LE",
            BinOp::LeEq => "LEEQ",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub children: Vec<Node>,
}

impl Node {
    fn number(num: &str) -> Self {
        Self {
            kind: NodeKind::Num(num.to_string()),
            children: vec![],
        }
    }
}

pub struct Parser {
    head: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { head: 0, tokens }
    }

    fn next(&mut self) -> &Token {
        let res = self.tokens.get(self.head).expect("unexpected eof");
        self.head += 1;
        res
    }

    fn peek(&self) -> &Token {
        let res = self.tokens.get(self.head).expect("unexpected eof");
        res
    }

    fn is_eof(&mut self) -> bool {
        self.head >= self.tokens.len()
    }

    fn consume(&mut self, target: &[TokenKind]) -> bool {
        if self.is_eof() {
            return false;
        }

        let l = target.len();
        if (0..l).all(|i| self.tokens[self.head + i].kind == target[i]) {
            self.head += l;
            true
        } else {
            false
        }
    }
}

impl Parser {
    pub fn parse(mut self) -> Node {
        self.program()
    }

    fn program(&mut self) -> Node {
        let mut res = Node {
            kind: NodeKind::Program,
            children: vec![],
        };

        loop {
            if self.is_eof() {
                break res;
            } else {
                res.children.push(self.stmt());
            }
        }
    }

    fn stmt(&mut self) -> Node {
        if self.consume(&[TokenKind::Return]) {
            let res = Node {
                kind: NodeKind::Return,
                children: vec![self.expr()],
            };
            assert!(self.consume(&[TokenKind::Semi]));
            res
        } else if self.peek().kind == TokenKind::LBrace {
            self.block()
        } else {
            let res = Node {
                kind: NodeKind::Stmt,
                children: vec![self.expr()],
            };
            self.consume(&[TokenKind::Semi]);
            res
        }
    }

    fn expr(&mut self) -> Node {
        if self.peek().kind == TokenKind::If {
            self.ifelse()
        } else if self.peek().kind == TokenKind::LBrace {
            self.block()
        } else {
            self.assign()
        }
    }

    fn block(&mut self) -> Node {
        self.consume(&[TokenKind::LBrace]);

        let mut res = Node {
            kind: NodeKind::Block,
            children: vec![],
        };

        loop {
            if self.consume(&[TokenKind::RBrace]) {
                break res;
            } else {
                res.children.push(self.stmt());
            }
        }
    }

    fn ifelse(&mut self) -> Node {
        self.consume(&[TokenKind::If]);
        let mut res = Node {
            kind: NodeKind::If,
            children: vec![self.expr(), self.block()],
        };

        if self.consume(&[TokenKind::Else]) {
            res.children.push(if self.peek().kind == TokenKind::If {
                self.ifelse()
            } else {
                self.block()
            });
        }

        res
    }

    fn assign(&mut self) -> Node {
        let res = self.equality();
        if self.consume(&[TokenKind::Eq]) {
            Node {
                kind: NodeKind::Assign,
                children: vec![res, self.expr()],
            }
        } else {
            res
        }
    }

    fn equality(&mut self) -> Node {
        let mut res = self.relational();

        loop {
            if self.consume(&[TokenKind::Eq, TokenKind::Eq]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Eq),
                    children: vec![res, self.relational()],
                }
            } else if self.consume(&[TokenKind::Bang, TokenKind::Eq]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Neq),
                    children: vec![res, self.relational()],
                }
            } else {
                break res;
            }
        }
    }

    fn relational(&mut self) -> Node {
        let mut res = self.add();

        loop {
            if self.consume(&[TokenKind::Lt, TokenKind::Eq]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::LeEq),
                    children: vec![res, self.add()],
                }
            } else if self.consume(&[TokenKind::Gt, TokenKind::Eq]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::LeEq),
                    children: vec![self.add(), res],
                }
            } else if self.consume(&[TokenKind::Lt]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Le),
                    children: vec![res, self.add()],
                }
            } else if self.consume(&[TokenKind::Gt]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Le),
                    children: vec![self.add(), res],
                }
            } else {
                break res;
            }
        }
    }

    fn add(&mut self) -> Node {
        let mut res = self.mul();

        loop {
            if self.consume(&[TokenKind::Plus]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Add),
                    children: vec![res, self.mul()],
                };
            } else if self.consume(&[TokenKind::Minus]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Sub),
                    children: vec![res, self.mul()],
                };
            } else {
                break res;
            }
        }
    }

    fn mul(&mut self) -> Node {
        let mut res = self.unary();

        loop {
            if self.consume(&[TokenKind::Star]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Mul),
                    children: vec![res, self.unary()],
                };
            } else if self.consume(&[TokenKind::Slash]) {
                res = Node {
                    kind: NodeKind::BinOp(BinOp::Div),
                    children: vec![res, self.unary()],
                };
            } else {
                break res;
            }
        }
    }

    fn unary(&mut self) -> Node {
        if self.consume(&[TokenKind::Minus]) {
            Node {
                kind: NodeKind::BinOp(BinOp::Sub),
                children: vec![Node::number("0"), self.primary()],
            }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Node {
        match self.next() {
            Token {
                kind: TokenKind::LParen,
                ..
            } => {
                let res = self.add();
                assert_eq!(self.next().kind, TokenKind::RParen);
                res
            }
            Token {
                kind: TokenKind::Ident,
                value: id,
                ..
            } => Node {
                kind: NodeKind::Ident(id.clone().unwrap()),
                children: vec![],
            },
            Token {
                kind: TokenKind::Num,
                value: num,
                ..
            } => Node {
                kind: NodeKind::Num(num.clone().unwrap()),
                children: vec![],
            },
            unknown => {
                panic!("unexpected {:?}", unknown)
            }
        }
    }
}

// impl Node {
// }

#[test]
fn feature() {
    let tokens = crate::lexer::tokenize("a=1; b=2; c=3;");
    dbg!(&tokens);

    let parser = Parser::new(tokens);
    dbg!(parser.parse());
}

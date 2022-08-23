use crate::lexer::{Token, TokenKind};

#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    Program,
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Le,
    LeEq,
    Assign,
    Stmt,
    Block,

    Return,

    Num(String),
    Ident(String),
}

impl ToString for NodeKind {
    fn to_string(&self) -> String {
        match self {
            NodeKind::Program => "PROGRAM",
            NodeKind::Add => "ADD",
            NodeKind::Sub => "SUB",
            NodeKind::Mul => "MUL",
            NodeKind::Div => "DIV",
            NodeKind::Eq => "EQ",
            NodeKind::Neq => "NEQ",
            NodeKind::Le => "LE",
            NodeKind::LeEq => "LEEQ",
            NodeKind::Assign => "ASSIGN",
            NodeKind::Stmt => "STMT",
            NodeKind::Block => "BLOCK",
            NodeKind::Return => "RETURN",
            NodeKind::Num(_) => "NUM",
            NodeKind::Ident(_) => "LOCAL",
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

    fn eof(&mut self) -> bool {
        self.head >= self.tokens.len()
    }

    fn consume(&mut self, target: &[TokenKind]) -> bool {
        if self.eof() {
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
            if self.eof() {
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
            self.consume(&[TokenKind::Semi]);
            res
        } else if self.consume(&[TokenKind::LBrace]) {
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
        self.assign()
    }

    fn assign(&mut self) -> Node {
        let res = self.equality();
        if self.consume(&[TokenKind::Eq]) {
            Node {
                kind: NodeKind::Assign,
                children: vec![res, self.equality()],
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
                    kind: NodeKind::Eq,
                    children: vec![res, self.relational()],
                }
            } else if self.consume(&[TokenKind::Bang, TokenKind::Eq]) {
                res = Node {
                    kind: NodeKind::Neq,
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
                    kind: NodeKind::LeEq,
                    children: vec![res, self.add()],
                }
            } else if self.consume(&[TokenKind::Gt, TokenKind::Eq]) {
                res = Node {
                    kind: NodeKind::LeEq,
                    children: vec![self.add(), res],
                }
            } else if self.consume(&[TokenKind::Lt]) {
                res = Node {
                    kind: NodeKind::Le,
                    children: vec![res, self.add()],
                }
            } else if self.consume(&[TokenKind::Gt]) {
                res = Node {
                    kind: NodeKind::Le,
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
                    kind: NodeKind::Add,
                    children: vec![res, self.mul()],
                };
            } else if self.consume(&[TokenKind::Minus]) {
                res = Node {
                    kind: NodeKind::Sub,
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
                    kind: NodeKind::Mul,
                    children: vec![res, self.unary()],
                };
            } else if self.consume(&[TokenKind::Slash]) {
                res = Node {
                    kind: NodeKind::Div,
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
                kind: NodeKind::Sub,
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

use crate::tokenizer::{Token, Tokenizer};

#[derive(Debug, PartialEq, Eq)]
pub struct Local {
    pub name: String,
    pub offset: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum NodeKind {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Le,
    LeEq,
    Assign,
    Semi,
    Num { value: String },
    Local(Local),
    Return,
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
}

impl Node {
    pub fn new(tokens: &mut Tokenizer) -> Self {
        Self::program(tokens)
    }

    fn program(tokens: &mut Tokenizer) -> Self {
        let mut res = Node {
            kind: NodeKind::Semi,
            lhs: Some(Box::new(Self::stmt(tokens))),
            rhs: None,
        };

        res.rhs = if tokens.eof() {
            None
        } else {
            tokens.consume(&Token::Operator(";"));
            if !tokens.eof() {
                Some(Box::new(Self::program(tokens)))
            } else {
                None
            }
        };

        res
    }

    fn stmt(tokens: &mut Tokenizer) -> Self {
        if tokens.consume(&Token::Return) {
            let res = Node {
                kind: NodeKind::Return,
                lhs: Some(Box::new(Self::expr(tokens))),
                rhs: None,
            };
            tokens.expect(&Token::Operator(";"));
            res
        } else {
            Self::expr(tokens)
        }
    }

    fn expr(tokens: &mut Tokenizer) -> Self {
        Self::assign(tokens)
    }

    fn assign(tokens: &mut Tokenizer) -> Self {
        let mut res = Self::equality(tokens);
        if tokens.consume(&Token::Operator("=")) {
            res = Node {
                kind: NodeKind::Assign,
                lhs: Some(Box::new(res)),
                rhs: Some(Box::new(Self::assign(tokens))),
            }
        }
        res
    }

    fn equality(tokens: &mut Tokenizer) -> Self {
        let mut res = Self::relational(tokens);

        loop {
            if tokens.consume(&Token::Operator("==")) {
                res = Node {
                    kind: NodeKind::Eq,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::relational(tokens))),
                }
            } else if tokens.consume(&Token::Operator("!=")) {
                res = Node {
                    kind: NodeKind::Neq,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::relational(tokens))),
                }
            } else {
                return res;
            }
        }
    }

    fn relational(tokens: &mut Tokenizer) -> Self {
        let mut res = Self::add(tokens);

        loop {
            if tokens.consume(&Token::Operator("<=")) {
                res = Node {
                    kind: NodeKind::LeEq,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::add(tokens))),
                }
            } else if tokens.consume(&Token::Operator(">=")) {
                res = Node {
                    kind: NodeKind::LeEq,
                    lhs: Some(Box::new(Self::add(tokens))),
                    rhs: Some(Box::new(res)),
                }
            } else if tokens.consume(&Token::Operator("<")) {
                res = Node {
                    kind: NodeKind::Le,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::add(tokens))),
                }
            } else if tokens.consume(&Token::Operator(">")) {
                res = Node {
                    kind: NodeKind::Le,
                    lhs: Some(Box::new(Self::add(tokens))),
                    rhs: Some(Box::new(res)),
                }
            } else {
                return res;
            }
        }
    }

    fn add(tokens: &mut Tokenizer) -> Self {
        let mut res = Self::mul(tokens);

        loop {
            if tokens.consume(&Token::Operator("+")) {
                res = Node {
                    kind: NodeKind::Add,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::mul(tokens))),
                };
            } else if tokens.consume(&Token::Operator("-")) {
                res = Node {
                    kind: NodeKind::Sub,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::mul(tokens))),
                };
            } else {
                return res;
            }
        }
    }

    fn mul(tokens: &mut Tokenizer) -> Self {
        let mut res = Self::unary(tokens);

        loop {
            if tokens.consume(&Token::Operator("*")) {
                res = Node {
                    kind: NodeKind::Mul,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::unary(tokens))),
                };
            } else if tokens.consume(&Token::Operator("/")) {
                res = Node {
                    kind: NodeKind::Div,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::unary(tokens))),
                };
            } else {
                return res;
            }
        }
    }

    fn unary(tokens: &mut Tokenizer) -> Self {
        if tokens.consume(&Token::Operator("+")) {
            Self::primary(tokens)
        } else if tokens.consume(&Token::Operator("-")) {
            Node {
                kind: NodeKind::Sub,
                lhs: Some(Box::new(Node {
                    kind: NodeKind::Num {
                        value: "0".to_string(),
                    },
                    lhs: None,
                    rhs: None,
                })),
                rhs: Some(Box::new(Self::primary(tokens))),
            }
        } else {
            Self::primary(tokens)
        }
    }

    fn primary(tokens: &mut Tokenizer) -> Self {
        match tokens.get().clone() {
            Token::Operator("(") => {
                tokens.consume(&Token::Operator("("));
                let res = Self::add(tokens);
                tokens.expect(&Token::Operator(")"));
                res
            }
            Token::Ident(id) => {
                tokens.consume(&Token::Ident(id.clone()));

                let l = tokens.variables.len();
                let offset = *tokens.variables.entry(id.clone()).or_insert((l + 1) * 8);

                Node {
                    kind: NodeKind::Local(Local { name: id, offset }),
                    lhs: None,
                    rhs: None,
                }
            }
            Token::Num(_) => Node {
                kind: NodeKind::Num {
                    value: tokens.expect_number(),
                },
                lhs: None,
                rhs: None,
            },
            Token::EOF => panic!("unexpected EOF"),
            unknown => {
                panic!("unexpected {:?}", unknown)
            }
        }
    }
}

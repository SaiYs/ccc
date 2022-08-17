#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Reserved(&'static str),
    Ident(char),
    Num(String),
    EOF,
}

#[derive(Debug)]
pub struct TokenList {
    tokens: Vec<Token>,
    head: usize,
}

impl TokenList {
    pub fn new(source: String) -> Self {
        let mut tokens = Vec::new();
        let source = source.chars().collect::<Vec<_>>();
        let mut cur = 0;
        while cur < source.len() {
            match source[cur] {
                ws if ws.is_ascii_whitespace() => cur += 1,
                d if d.is_digit(10) => {
                    let l = (cur..)
                        .take_while(|&x| x < source.len() && source[x].is_digit(10))
                        .count();
                    let n = source[cur..cur + l].iter().collect::<String>();

                    tokens.push(Token::Num(n));
                    cur += l;
                }
                id if id.is_ascii_lowercase() => {
                    tokens.push(Token::Ident(id));
                    cur += 1;
                }
                _ => {
                    if source[cur..].starts_with(&['=', '=']) {
                        tokens.push(Token::Reserved("=="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['!', '=']) {
                        tokens.push(Token::Reserved("!="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['<', '=']) {
                        tokens.push(Token::Reserved("<="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['>', '=']) {
                        tokens.push(Token::Reserved(">="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['<']) {
                        tokens.push(Token::Reserved("<"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['>']) {
                        tokens.push(Token::Reserved(">"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['+']) {
                        tokens.push(Token::Reserved("+"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['-']) {
                        tokens.push(Token::Reserved("-"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['*']) {
                        tokens.push(Token::Reserved("*"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['/']) {
                        tokens.push(Token::Reserved("/"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['(']) {
                        tokens.push(Token::Reserved("("));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&[')']) {
                        tokens.push(Token::Reserved(")"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['=']) {
                        tokens.push(Token::Reserved("="));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&[';']) {
                        tokens.push(Token::Reserved(";"));
                        cur += 1;
                    }
                }
            }
        }

        tokens.push(Token::EOF);

        TokenList { tokens, head: 0 }
    }

    fn get(&self) -> &Token {
        &self.tokens[self.head]
    }

    fn consume(&mut self, expected: Token) -> bool {
        let f = self.get() == &expected;
        if f {
            self.head += 1;
        }
        f
    }

    fn expect(&mut self, expected: Token) -> Token {
        match self.get() {
            actual if actual == &expected => {
                self.head += 1;
                expected
            }
            _ => panic!("unexpected token"),
        }
    }

    fn expect_number(&mut self) -> String {
        let res = match self.get() {
            Token::Num(n) => n.clone(),
            _ => panic!("expected number but found not number"),
        };
        self.head += 1;
        res
    }

    fn eof(&self) -> bool {
        self.get() == &Token::EOF
    }
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
    Local { offset: u8 },
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
}

impl Node {
    pub fn new(tokens: &mut TokenList) -> Self {
        Self::program(tokens)
    }

    fn program(tokens: &mut TokenList) -> Self {
        let mut res = Node {
            kind: NodeKind::Semi,
            lhs: Some(Box::new(Self::stmt(tokens))),
            rhs: None,
        };
        tokens.consume(Token::Reserved(";"));
        res.rhs = if tokens.eof() {
            None
        } else {
            Some(Box::new(Self::program(tokens)))
        };

        res
    }

    fn stmt(tokens: &mut TokenList) -> Self {
        Self::expr(tokens)
    }

    fn expr(tokens: &mut TokenList) -> Self {
        Self::assign(tokens)
    }

    fn assign(tokens: &mut TokenList) -> Self {
        let mut res = Self::equality(tokens);
        if tokens.consume(Token::Reserved("=")) {
            res = Node {
                kind: NodeKind::Assign,
                lhs: Some(Box::new(res)),
                rhs: Some(Box::new(Self::assign(tokens))),
            }
        }
        res
    }

    fn equality(tokens: &mut TokenList) -> Self {
        let mut res = Self::relational(tokens);

        loop {
            if tokens.consume(Token::Reserved("==")) {
                res = Node {
                    kind: NodeKind::Eq,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::relational(tokens))),
                }
            } else if tokens.consume(Token::Reserved("!=")) {
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

    fn relational(tokens: &mut TokenList) -> Self {
        let mut res = Self::add(tokens);

        loop {
            if tokens.consume(Token::Reserved("<=")) {
                res = Node {
                    kind: NodeKind::LeEq,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::add(tokens))),
                }
            } else if tokens.consume(Token::Reserved(">=")) {
                res = Node {
                    kind: NodeKind::LeEq,
                    lhs: Some(Box::new(Self::add(tokens))),
                    rhs: Some(Box::new(res)),
                }
            } else if tokens.consume(Token::Reserved("<")) {
                res = Node {
                    kind: NodeKind::Le,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::add(tokens))),
                }
            } else if tokens.consume(Token::Reserved(">")) {
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

    fn add(tokens: &mut TokenList) -> Self {
        let mut res = Self::mul(tokens);

        loop {
            if tokens.consume(Token::Reserved("+")) {
                res = Node {
                    kind: NodeKind::Add,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::mul(tokens))),
                };
            } else if tokens.consume(Token::Reserved("-")) {
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

    fn mul(tokens: &mut TokenList) -> Self {
        let mut res = Self::unary(tokens);

        loop {
            if tokens.consume(Token::Reserved("*")) {
                res = Node {
                    kind: NodeKind::Mul,
                    lhs: Some(Box::new(res)),
                    rhs: Some(Box::new(Self::unary(tokens))),
                };
            } else if tokens.consume(Token::Reserved("/")) {
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

    fn unary(tokens: &mut TokenList) -> Self {
        if tokens.consume(Token::Reserved("+")) {
            Self::primary(tokens)
        } else if tokens.consume(Token::Reserved("-")) {
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

    fn primary(tokens: &mut TokenList) -> Self {
        match tokens.get() {
            Token::Reserved("(") => {
                tokens.consume(Token::Reserved("("));
                let res = Self::add(tokens);
                tokens.expect(Token::Reserved(")"));
                res
            }
            Token::Ident(id) => {
                let id = *id;
                tokens.consume(Token::Ident(id));

                Node {
                    kind: NodeKind::Local {
                        offset: (id as u8 - b'a' + 1) * 8,
                    },
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
            _ => panic!("unexpected EOF"),
        }
        // if tokens.consume(Token::Reserved("(")) {
        //     let inner = Self::add(tokens);
        //     tokens.expect(Token::Reserved(")"));
        //     inner
        // } else {
        //     Node {
        //         kind: NodeKind::Num {
        //             value: tokens.expect_number(),
        //         },
        //         lhs: None,
        //         rhs: None,
        //     }
        // }
    }
}

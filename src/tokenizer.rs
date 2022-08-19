use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Operator(&'static str),
    Ident(String),
    Num(String),
    Return,
    EOF,
}

#[derive(Debug)]
pub struct Tokenizer {
    tokens: Vec<Token>,
    head: usize,
    pub variables: HashMap<String, usize>,
}

impl Tokenizer {
    pub fn new(source: String) -> Self {
        let mut tokens = Vec::new();
        let source = source.chars().collect::<Vec<_>>();
        let mut cur = 0;

        while cur < source.len() {
            match source[cur] {
                // whitespace
                ws if ws.is_ascii_whitespace() => cur += 1,
                // integer
                d if d.is_digit(10) => {
                    let l = (cur..)
                        .take_while(|&x| x < source.len() && source[x].is_digit(10))
                        .count();
                    let n = source[cur..cur + l].iter().collect::<String>();

                    tokens.push(Token::Num(n));
                    cur += l;
                }
                // return
                _ if source[cur..].starts_with(&"return".chars().collect::<Vec<_>>()) => {
                    tokens.push(Token::Return);
                    cur += 6;
                }
                // identifier
                id if id.is_ascii_lowercase() => {
                    let l = (cur..)
                        .take_while(|&x| x < source.len() && source[x].is_ascii_alphanumeric())
                        .count();
                    let id = source[cur..cur + l].iter().collect::<String>();
                    tokens.push(Token::Ident(id));
                    cur += l;
                }
                // operators
                _ => {
                    if source[cur..].starts_with(&['=', '=']) {
                        tokens.push(Token::Operator("=="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['!', '=']) {
                        tokens.push(Token::Operator("!="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['<', '=']) {
                        tokens.push(Token::Operator("<="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['>', '=']) {
                        tokens.push(Token::Operator(">="));
                        cur += 2;
                    }
                    if source[cur..].starts_with(&['<']) {
                        tokens.push(Token::Operator("<"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['>']) {
                        tokens.push(Token::Operator(">"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['+']) {
                        tokens.push(Token::Operator("+"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['-']) {
                        tokens.push(Token::Operator("-"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['*']) {
                        tokens.push(Token::Operator("*"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['/']) {
                        tokens.push(Token::Operator("/"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['(']) {
                        tokens.push(Token::Operator("("));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&[')']) {
                        tokens.push(Token::Operator(")"));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&['=']) {
                        tokens.push(Token::Operator("="));
                        cur += 1;
                    }
                    if source[cur..].starts_with(&[';']) {
                        tokens.push(Token::Operator(";"));
                        cur += 1;
                    }
                }
            }
        }

        tokens.push(Token::EOF);

        Tokenizer {
            tokens,
            head: 0,
            variables: HashMap::new(),
        }
    }

    pub fn get(&self) -> &Token {
        &self.tokens[self.head]
    }

    pub fn consume(&mut self, expected: &Token) -> bool {
        let f = self.get() == expected;
        if f {
            self.head += 1;
        }
        f
    }

    pub fn expect(&mut self, expected: &Token) -> Token {
        match self.get() {
            actual if actual == expected => {
                self.head += 1;
                expected.clone()
            }
            _ => panic!("unexpected token"),
        }
    }

    pub fn expect_number(&mut self) -> String {
        let res = match self.get() {
            Token::Num(n) => n.clone(),
            _ => panic!("expected number but found not number"),
        };
        self.head += 1;
        res
    }

    pub fn eof(&self) -> bool {
        self.get() == &Token::EOF
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Reserved(String),
    Num(u32),
    EOF,
}

#[derive(Debug)]
pub struct TokenList {
    tokens: Vec<Token>,
    head: usize,
}

impl TokenList {
    fn new(source: String) -> Self {
        TokenList {
            tokens: tokenize(source),
            head: 0,
        }
    }

    fn expect_number(&mut self) -> u32 {
        match self.tokens[self.head] {
            Token::Num(n) => {
                self.head += 1;
                n
            }
            _ => panic!("expected number but found not number"),
        }
    }

    fn expect_reserved(&mut self) -> String {
        match self.tokens[self.head] {
            Token::Reserved(ref s) => {
                self.head += 1;
                s.clone()
            }
            _ => panic!("expected reserved but found not reserved"),
        }
    }

    fn eof(&self) -> bool {
        self.tokens[self.head] == Token::EOF
    }
}

fn tokenize(source: String) -> Vec<Token> {
    let mut res = Vec::new();
    let mut cur = None;
    for c in source.chars() {
        if let Some(m) = c.to_digit(10) {
            if let Some(n) = cur.as_mut() {
                *n = *n * 10 + m;
            } else {
                cur = Some(m);
            }
        } else {
            if let Some(n) = cur {
                res.push(Token::Num(n));
                cur = None;
            }

            if c.is_ascii_whitespace() {
                continue;
            }
            if c == '+' || c == '-' {
                res.push(Token::Reserved(c.to_string()));
            }
        }
    }
    if let Some(n) = cur {
        res.push(Token::Num(n));
    }

    res.push(Token::EOF);
    res
}

fn main() {
    let mut args = std::env::args();
    if args.len() != 2 {
        panic!("invalid args");
    }

    let source = args.nth(1).unwrap();
    let mut tokens = TokenList::new(source);

    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    println!("    mov rax, {}", tokens.expect_number());

    while !tokens.eof() {
        let op = tokens.expect_reserved();
        if op == "+" {
            println!("    add rax, {}", tokens.expect_number());
        } else if op == "-" {
            println!("    sub rax, {}", tokens.expect_number());
        }
    }

    println!("    ret");
}

#[test]
fn test_tokenize() {
    let s = "1 + 2 + 4 - 1";
    let tokens = tokenize(s.to_string());
    dbg!(tokens);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenKind {
    // punctuations
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,
    /// &
    And,
    /// |
    Or,
    /// ^
    Caret,
    /// <
    Lt,
    /// >
    Gt,
    /// (
    LParen,
    /// )
    RParen,
    /// {
    LBrace,
    /// }
    RBrace,
    /// [
    LBlanket,
    /// ]
    RBlanket,
    /// =
    Eq,
    /// !
    Bang,
    /// ?
    Question,
    /// :
    Colon,
    /// ;
    Semi,
    /// ,
    Comma,
    /// .
    Dot,

    // keywords
    Fn,
    If,
    Else,
    For,
    While,
    Loop,
    Return,

    /// identifier
    Ident,
    /// number literal
    Num,
    /// whitespace
    Whitespace,
    // EOF,
}

impl ToString for TokenKind {
    fn to_string(&self) -> String {
        match self {
            TokenKind::Plus => "+",
            TokenKind::Minus => "-",
            TokenKind::Star => "*",
            TokenKind::Slash => "/",
            TokenKind::Percent => "%",
            TokenKind::And => "&",
            TokenKind::Or => "|",
            TokenKind::Caret => "^",
            TokenKind::Lt => "<",
            TokenKind::Gt => ">",
            TokenKind::LParen => "(",
            TokenKind::RParen => ")",
            TokenKind::LBrace => "{",
            TokenKind::RBrace => "}",
            TokenKind::LBlanket => "[",
            TokenKind::RBlanket => "]",
            TokenKind::Eq => "=",
            TokenKind::Bang => "!",
            TokenKind::Question => "?",
            TokenKind::Colon => ":",
            TokenKind::Semi => ";",
            TokenKind::Comma => ",",
            TokenKind::Dot => ".",

            TokenKind::Fn => "fn",
            TokenKind::If => "if",
            TokenKind::Else => "else",
            TokenKind::For => "for",
            TokenKind::While => "while",
            TokenKind::Loop => "loop",
            TokenKind::Return => "return",

            TokenKind::Ident => "ident",
            TokenKind::Num => "number",
            TokenKind::Whitespace => "",
        }
        .to_string()
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub value: Option<String>,
    pub pos: (usize, usize),
}

fn is_id_head(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_id_body(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '_')
}

fn is_keyword(id: &str) -> bool {
    matches!(id, "fn" | "if" | "else" | "loop" | "return")
}

fn to_keyword(id: &str) -> Option<TokenKind> {
    match id {
        "fn" => Some(TokenKind::Fn),
        "if" => Some(TokenKind::If),
        "else" => Some(TokenKind::Else),
        "loop" => Some(TokenKind::Loop),
        "return" => Some(TokenKind::Return),
        _ => None,
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut cursor = Cursor::new(input);
    std::iter::from_fn(move || {
        if cursor.is_eof() {
            None
        } else {
            Some(cursor.token())
        }
    })
    .filter(|x| x.kind != TokenKind::Whitespace)
    .collect()
}

const EOF_CHAR: char = '\0';

struct Cursor<'a> {
    pos: usize,
    last: usize,
    chars: std::str::Chars<'a>,
}

impl<'a> Cursor<'a> {
    fn new(source: &'a str) -> Self {
        let chars = source.chars();
        Self {
            pos: 0,
            last: 0,
            chars,
        }
    }

    fn update_pos(&mut self) -> (usize, usize) {
        let res = (self.last, self.pos);
        self.last = self.pos;
        res
    }

    fn first(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn bump(&mut self) -> Option<char> {
        self.pos += 1;
        self.chars.next()
    }

    fn consume(&mut self, target: &str) {
        let len = target.len();
        self.pos += len;
        let consumed = self.chars.by_ref().take(len).collect::<String>();
        debug_assert!(consumed == target)
    }

    fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    fn token(&mut self) -> Token {
        match self.first() {
            whitespace if whitespace.is_ascii_whitespace() => {
                while self.first().is_ascii_whitespace() {
                    self.bump();
                }
                Token {
                    kind: TokenKind::Whitespace,
                    value: None,
                    pos: self.update_pos(),
                }
            }

            // identity or keyword
            c if is_id_head(c) => {
                let id = self
                    .chars
                    .clone()
                    .take_while(|&x| is_id_body(x))
                    .collect::<String>();
                self.consume(&id);

                if is_keyword(&id) {
                    Token {
                        kind: to_keyword(&id).unwrap(),
                        value: None,
                        pos: self.update_pos(),
                    }
                } else {
                    Token {
                        kind: TokenKind::Ident,
                        value: Some(id),
                        pos: self.update_pos(),
                    }
                }
            }

            // numeric literal
            c if c.is_ascii_digit() => {
                let num: String = self
                    .chars
                    .clone()
                    .take_while(|&x| x.is_ascii_digit())
                    .collect::<String>()
                    .parse()
                    .unwrap();
                self.consume(&num);
                Token {
                    kind: TokenKind::Num,
                    value: Some(num),
                    pos: self.update_pos(),
                }
            }

            // punctuations
            '=' => {
                self.bump();
                Token {
                    kind: TokenKind::Eq,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '+' => {
                self.bump();
                Token {
                    kind: TokenKind::Plus,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '-' => {
                self.bump();
                Token {
                    kind: TokenKind::Minus,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '*' => {
                self.bump();
                Token {
                    kind: TokenKind::Star,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '/' => {
                self.bump();
                Token {
                    kind: TokenKind::Slash,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '&' => {
                self.bump();
                Token {
                    kind: TokenKind::And,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '|' => {
                self.bump();
                Token {
                    kind: TokenKind::Or,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '^' => {
                self.bump();
                Token {
                    kind: TokenKind::Caret,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '<' => {
                self.bump();
                Token {
                    kind: TokenKind::Lt,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '>' => {
                self.bump();
                Token {
                    kind: TokenKind::Gt,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '(' => {
                self.bump();
                Token {
                    kind: TokenKind::LParen,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            ')' => {
                self.bump();
                Token {
                    kind: TokenKind::RParen,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '[' => {
                self.bump();
                Token {
                    kind: TokenKind::LBlanket,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            ']' => {
                self.bump();
                Token {
                    kind: TokenKind::RBlanket,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '{' => {
                self.bump();
                Token {
                    kind: TokenKind::LBrace,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '}' => {
                self.bump();
                Token {
                    kind: TokenKind::RBrace,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            ',' => {
                self.bump();
                Token {
                    kind: TokenKind::Comma,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '.' => {
                self.bump();
                Token {
                    kind: TokenKind::Dot,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '!' => {
                self.bump();
                Token {
                    kind: TokenKind::Bang,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            '?' => {
                self.bump();
                Token {
                    kind: TokenKind::Question,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            ':' => {
                self.bump();
                Token {
                    kind: TokenKind::Colon,
                    value: None,
                    pos: self.update_pos(),
                }
            }
            ';' => {
                self.bump();
                Token {
                    kind: TokenKind::Semi,
                    value: None,
                    pos: self.update_pos(),
                }
            }

            unknown => panic!("unexpected {:?} at {}", unknown, self.pos),
        }
    }
}

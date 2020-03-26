pub struct Scanner<'a> {
    source: &'a String,
    start: std::iter::Peekable<std::str::CharIndices<'a>>, // TODO - use iterators? then don't need the source string?
    current: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: i32,
}

// TODO - named struct for every token type seems dumb. How can you just embed these on each type?
pub struct TokenInfo<'a> {
    string: &'a str, // The slice that actually holds the string containing the token
    line: i32,
}

pub enum Token<'a> {
    // Single-character tokens.
    TokenLeftParen(TokenInfo<'a>),
    TokenRightParen(TokenInfo<'a>),
    TokenLeftBrace(TokenInfo<'a>),
    TokenRightBrace(TokenInfo<'a>),
    TokenComma(TokenInfo<'a>),
    TokenDot(TokenInfo<'a>),
    TokenMinus(TokenInfo<'a>),
    TokenPlus(TokenInfo<'a>),
    TokenSemicolon(TokenInfo<'a>),
    TokenSlash(TokenInfo<'a>),
    TokenStar(TokenInfo<'a>),

    // One or two character tokens.
    TokenBang(TokenInfo<'a>),
    TokenBangEqual(TokenInfo<'a>),
    TokenEqual(TokenInfo<'a>),
    TokenEqualEqual(TokenInfo<'a>),
    TokenGreater(TokenInfo<'a>),
    TokenGreaterEqual(TokenInfo<'a>),
    TokenLess(TokenInfo<'a>),
    TokenLessEqual(TokenInfo<'a>),

    // Literals.
    TokenIdentifier(TokenInfo<'a>),
    TokenString(TokenInfo<'a>),
    TokenNumber(TokenInfo<'a>),

    // Keywords.
    TokenAnd(TokenInfo<'a>),
    TokenClass(TokenInfo<'a>),
    TokenElse(TokenInfo<'a>),
    TokenFalse(TokenInfo<'a>),
    TokenFor(TokenInfo<'a>),
    TokenFun(TokenInfo<'a>),
    TokenIf(TokenInfo<'a>),
    TokenNil(TokenInfo<'a>),
    TokenOr(TokenInfo<'a>),
    TokenPrint(TokenInfo<'a>),
    TokenReturn(TokenInfo<'a>),
    TokenSuper(TokenInfo<'a>),
    TokenThis(TokenInfo<'a>),
    TokenTrue(TokenInfo<'a>),
    TokenVar(TokenInfo<'a>),
    TokenWhile(TokenInfo<'a>),

    TokenError(TokenInfo<'a>),
    TokenEof(TokenInfo<'a>),
}

// TODO - this seems like a macro to me? or template function...?
// C Code:
// static Token makeToken(TokenType type) {
//     Token token;
//     token.type = type;
//     token.start = scanner.start;
//     token.length = (int)(scanner.current - scanner.start);
//     token.line = scanner.line;
//
//     return token;
// }
//
// TODO - there's no way this is the right way to create a string slice from iterators.
macro_rules! make_token {
    ($self:ident, $token_type:tt) => {
        Token::$token_type(TokenInfo {
            string: &$self.source[$self.start.peek().unwrap().0..$self.current.peek().unwrap().0],
            line: $self.line,
        })
    };
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a std::string::String) -> Self {
        Scanner {
            source: source,
            start: source.char_indices().peekable(),
            current: source.char_indices().peekable(),
            line: 1,
        }
    }

    pub fn is_at_end(&self) -> bool {
        return self.current.peek() == None;
    }

    fn advance(&mut self) -> char {
        self.current.next().unwrap().1
    }

    fn match_character(&mut self, character: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.current.peek().unwrap().1 != character {
            return false;
        }

        self.current.next();
        return true;
    }

    fn peek(&self) -> char {
        self.current.peek().unwrap().1
    }

    fn peek_next(&self) -> char {
        let next = self.current.clone().next();

        if next == None {
            return '\0';
        }

        next.unwrap().1
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.is_at_end() {
                return;
            }

            let c = self.peek();

            match c {
                '\n' => {
                    self.line += 1;
                    self.advance();
                }
                x if x.is_whitespace() => {
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == '/' {
                        // Comments eat everything until the next line.
                        while self.peek() != '\n' && !self.is_at_end() {
                            self.advance();
                        }
                    } else {
                        return;
                    }
                }
                _ => return,
            }
        }
    }

    fn is_digit(c: char) -> bool {
        return c >= '0' && c <= '9';
    }

    fn is_alpha(c: char) -> bool {
        return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_';
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current;

        if self.is_at_end() {
            return make_token!(self, TokenEof);
        }

        let c = self.advance();

        match c {
            '(' => return make_token!(self, TokenLeftParen),
            ')' => return make_token!(self, TokenRightParen),
            '{' => return make_token!(self, TokenLeftBrace),
            '}' => return make_token!(self, TokenRightBrace),
            ';' => return make_token!(self, TokenSemicolon),
            ',' => return make_token!(self, TokenComma),
            '.' => return make_token!(self, TokenDot),
            '-' => return make_token!(self, TokenMinus),
            '+' => return make_token!(self, TokenPlus),
            '/' => return make_token!(self, TokenSlash),
            '*' => return make_token!(self, TokenStar),
            // TODO - Probably can macro these double character matches too.
            '!' => {
                if self.match_character('=') {
                    return make_token!(self, TokenBangEqual);
                } else {
                    return make_token!(self, TokenBang);
                }
            }
            '=' => {
                if self.match_character('=') {
                    return make_token!(self, TokenEqualEqual);
                } else {
                    return make_token!(self, TokenBang);
                }
            }
            '<' => {
                if self.match_character('=') {
                    return make_token!(self, TokenLessEqual);
                } else {
                    return make_token!(self, TokenLess);
                }
            }
            '>' => {
                if self.match_character('=') {
                    return make_token!(self, TokenGreaterEqual);
                } else {
                    return make_token!(self, TokenGreater);
                }
            }
            '"' => {
                return self.make_string_token();
            }
            x if Scanner::is_digit(x) => {
                return self.make_number_token();
            }
            x if Scanner::is_alpha(x) => {
                return self.make_identifier_token();
            }
            _ => {}
        }

        self.make_error_token("Unexpected character.")
    }

    fn make_error_token(&self, string: &'a str) -> Token {
        Token::TokenError(TokenInfo {
            string: string,
            line: self.line,
        })
    }

    fn make_string_token(&mut self) -> Token {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }

            self.advance();
        }

        if self.is_at_end() {
            return self.make_error_token("Unterminated string.");
        }

        // The closing quote.
        self.advance();

        make_token!(self, TokenString)
    }

    fn make_number_token(&mut self) -> Token {
        while self.peek().is_numeric() {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && self.peek_next().is_numeric() {
            // Consume the ".".
            self.advance();

            while self.peek().is_numeric() {
                self.advance();
            }
        }

        make_token!(self, TokenNumber)
    }

    // Check if the identifier we matched is a keyword.
    fn check_keyword(iter: std::iter::Peekable<std::str::CharIndices<'a>>, rest: &str) -> bool {
        // First check length. If they don't match, can't possibly match.
        if (iter.count()) != rest.len() {
            return false;
        }

        // Check each character if they match.
        for c in rest.chars() {
            if c != iter.next().unwrap().1 {
                return false;
            }
        }

        // Everything matched, so it's a keyword.
        true
    }

    fn make_identifier_token(&mut self) -> Token {
        while Scanner::is_alpha(self.peek()) || Scanner::is_digit(self.peek()) {
            self.advance();
        }

        // Makes a keyword token if matches, otherwise returns an identifier token.
        // TODO - seems to me this could be a closure? or maybe not because of Token Type?
        macro_rules! make_keyword {
            ($iter:ident, $rest:expr, $token_type:tt) => {
                if Scanner::check_keyword($iter, $rest) {
                    return make_token!(self, TokenAnd);
                } else {
                    return make_token!(self, TokenIdentifier);
                }
            };
        }

        // Produce a token with the proper type.
        let mut iter = self.start.clone();
        match iter.next().unwrap().1 {
            'a' => make_keyword!(iter, "nd", TokenAnd),
            'c' => make_keyword!(iter, "lass", TokenClass),
            'e' => make_keyword!(iter, "lse", TokenElse),
            'f' => {
                let next = iter.next();
                match next {
                    Some(x) => match x.1 {
                        'a' => make_keyword!(iter, "lse", TokenFalse),
                        'o' => make_keyword!(iter, "r", TokenFor),
                        'u' => make_keyword!(iter, "n", TokenFun),
                    },
                    None => {}
                }
            }
            'i' => make_keyword!(iter, "f", TokenIf),
            'n' => make_keyword!(iter, "il", TokenNil),
            'o' => make_keyword!(iter, "r", TokenOr),
            'p' => make_keyword!(iter, "rint", TokenPrint),
            'r' => make_keyword!(iter, "eturn", TokenReturn),
            's' => make_keyword!(iter, "uper", TokenSuper),
            't' => {
                let next = iter.next();
                match next {
                    Some(x) => match x.1 {
                        'h' => make_keyword!(iter, "is", TokenThis),
                        'r' => make_keyword!(iter, "ue", TokenTrue),
                    },
                    None => {}
                }
            }
            'v' => make_keyword!(iter, "ar", TokenVar),
            'w' => make_keyword!(iter, "hile", TokenWhile),
        }

        make_token!(self, TokenIdentifier)
    }
}

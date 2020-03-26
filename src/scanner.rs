pub struct Scanner<'a> {
    source: &'a String,
    start: std::iter::Peekable<std::str::CharIndices<'a>>, // TODO - use iterators? then don't need the source string?
    current: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: i32,
}

#[derive(Debug)]
pub enum TokenType {
    // Single-character tokens.
    TokenLeftParen,
    TokenRightParen,
    TokenLeftBrace,
    TokenRightBrace,
    TokenComma,
    TokenDot,
    TokenMinus,
    TokenPlus,
    TokenSemicolon,
    TokenSlash,
    TokenStar,

    // One or two character tokens.
    TokenBang,
    TokenBangEqual,
    TokenEqual,
    TokenEqualEqual,
    TokenGreater,
    TokenGreaterEqual,
    TokenLess,
    TokenLessEqual,

    // Literals.
    TokenIdentifier,
    TokenString,
    TokenNumber,

    // Keywords.
    TokenAnd,
    TokenClass,
    TokenElse,
    TokenFalse,
    TokenFor,
    TokenFun,
    TokenIf,
    TokenNil,
    TokenOr,
    TokenPrint,
    TokenReturn,
    TokenSuper,
    TokenThis,
    TokenTrue,
    TokenVar,
    TokenWhile,

    TokenError,
    TokenEof,
}

// TODO - named struct for every token type seems dumb. How can you just embed these on each type?
pub struct Token<'a> {
    pub string: &'a str, // The slice that actually holds the string containing the token
    pub line: i32,
    pub token_type: TokenType,
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
// macro_rules! make_token {
//     ($self:ident, $token_type:tt) => {
//         Token::$token_type(TokenInfo {
//             string: &$self.source[$self.start.peek().unwrap().0..$self.current.peek().unwrap().0],
//             line: $self.line,
//         })
//     };
// }

// TODO - there's no way this is the right way to create a string slice from iterators.
impl<'a> Scanner<'a> {
    // TODO - self is mut why? cause peek is mut?
    fn make_token(&mut self, token_type: TokenType) -> Token {
        Token {
            string: &self.source[self.start.peek().unwrap().0..self.current.peek().unwrap().0],
            line: self.line,
            token_type: token_type,
        }
    }

    pub fn new(source: &'a std::string::String) -> Self {
        Scanner {
            source: source,
            start: source.char_indices().peekable(),
            current: source.char_indices().peekable(),
            line: 1,
        }
    }

    pub fn is_at_end(&mut self) -> bool {
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

    fn peek(&mut self) -> char {
        self.current.peek().unwrap().1
    }

    fn peek_next(&self) -> char {
        let next = self.current.clone().skip(1).next();

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

        self.start = self.current.clone();

        if self.is_at_end() {
            return Token {
                string: "",
                line: self.line,
                token_type: TokenType::TokenEof,
            };

            // calling common make_token panics, because we're at the end for
            // the iterators (duh)
            //
            // return self.make_token(TokenType::TokenEof);
            //
            // Instead, scan token should return an iterator/option. Basically
            // the scanner should be an iterator that takes a string and output
            // is Iterator<Item = Token>. Then next_token can return an option
            // and there can't be a misuse of it.
            //
        }

        let c = self.advance();

        match c {
            '(' => return self.make_token(TokenType::TokenLeftParen),
            ')' => return self.make_token(TokenType::TokenRightParen),
            '{' => return self.make_token(TokenType::TokenLeftBrace),
            '}' => return self.make_token(TokenType::TokenRightBrace),
            ';' => return self.make_token(TokenType::TokenSemicolon),
            ',' => return self.make_token(TokenType::TokenComma),
            '.' => return self.make_token(TokenType::TokenDot),
            '-' => return self.make_token(TokenType::TokenMinus),
            '+' => return self.make_token(TokenType::TokenPlus),
            '/' => return self.make_token(TokenType::TokenSlash),
            '*' => return self.make_token(TokenType::TokenStar),
            // TODO - Probably can macro these double character matches too.
            '!' => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenBangEqual);
                } else {
                    return self.make_token(TokenType::TokenBang);
                }
            }
            '=' => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenEqualEqual);
                } else {
                    return self.make_token(TokenType::TokenEqual);
                }
            }
            '<' => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenLessEqual);
                } else {
                    return self.make_token(TokenType::TokenLess);
                }
            }
            '>' => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenGreaterEqual);
                } else {
                    return self.make_token(TokenType::TokenGreater);
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
        Token {
            string: string,
            line: self.line,
            token_type: TokenType::TokenError,
        }
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

        self.make_token(TokenType::TokenString)
    }

    fn make_number_token(&mut self) -> Token {
        while self.peek().is_numeric() {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == '.' && Scanner::is_digit(self.peek_next()) {
            // Consume the ".".
            self.advance();

            while self.peek().is_numeric() {
                self.advance();
            }
        }

        self.make_token(TokenType::TokenNumber)
    }

    // Check if the identifier we matched is a keyword.
    fn check_keyword(mut iter: std::iter::Peekable<std::str::CharIndices<'a>>, rest: &str) -> bool {
        // First check length. If they don't match, can't possibly match.
        // TODO - how to check length without iterating through iterator, impossible??
        if (iter.clone().count()) != rest.len() {
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
                    return self.make_token(TokenType::$token_type);
                } else {
                    return self.make_token(TokenType::TokenIdentifier);
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
                        _ => (),
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
                        _ => (),
                    },
                    None => {}
                }
            }
            'v' => make_keyword!(iter, "ar", TokenVar),
            'w' => make_keyword!(iter, "hile", TokenWhile),
            _ => (),
        }

        self.make_token(TokenType::TokenIdentifier)
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;

    #[test]

    fn peek_next_test() {
        let string = String::from("1.2");
        let mut scanner = Scanner::new(&string);

        assert_eq!(scanner.peek(), '1');
        assert_eq!(scanner.peek_next(), '.');

        assert_eq!(scanner.advance(), '1');
        assert_eq!(scanner.peek(), '.');
        assert_eq!(scanner.peek_next(), '2');
    }
}

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

trait IsLoxDigit {
    fn is_lox_digit(&self) -> bool;
}

impl IsLoxDigit for char {
    fn is_lox_digit(&self) -> bool {
        *self >= '0' && *self <= '9'
    }
}

impl IsLoxDigit for Option<char> {
    fn is_lox_digit(&self) -> bool {
        if let Some(c) = self {
            return c.is_lox_digit();
        }
        false
    }
}

trait IsLoxAlpha {
    fn is_lox_alpha(&self) -> bool;
}

impl IsLoxAlpha for char {
    fn is_lox_alpha(&self) -> bool {
        (*self >= 'a' && *self <= 'z') || (*self >= 'A' && *self <= 'Z') || *self == '_'
    }
}

impl IsLoxAlpha for Option<char> {
    fn is_lox_alpha(&self) -> bool {
        if let Some(c) = self {
            return c.is_lox_alpha();
        }
        false
    }
}

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

    // All these "char" returning functions should return options. Then no need for unwrap!
    // and its more rusty
    //
    // Really, this is almost like next, but not quite?
    fn advance(&mut self) -> Option<char> {
        // Kind of cludgy, but the caller really doesn't want the usize. But we
        // need it to construct the string slice later.
        match self.current.next() {
            Some((_, x)) => Option::Some(x),
            None => None,
        }
    }

    fn match_character(&mut self, c: char) -> bool {
        match self.peek() {
            Some(x) if x == c => {
                self.current.next();
                true
            }
            _ => false,
        }
    }

    fn peek(&mut self) -> Option<char> {
        match self.current.peek() {
            Some((_, x)) => Option::Some(*x),
            None => None,
        }
    }

    fn peek_next(&self) -> Option<char> {
        let next = self.current.clone().nth(1);

        match next {
            Some((_, x)) => Option::Some(x),
            None => None,
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            match (self.peek(), self.peek_next()) {
                (Some('\n'), _) => {
                    self.line += 1;
                    self.advance();
                }
                (Some(x), _) if x.is_whitespace() => {
                    self.advance();
                }
                (Some('/'), Some('/')) => loop {
                    match self.peek() {
                        Some('\n') => {
                            break;
                        }
                        Some(_) => {
                            self.advance();
                        }
                        None => return,
                    }
                },
                _ => return,
            }
        }
    }

    pub fn scan_token(&mut self) -> Token {
        self.skip_whitespace();

        self.start = self.current.clone();

        // Instead, scan token should return an iterator/option. Basically
        // the scanner should be an iterator that takes a string and output
        // is Iterator<Item = Token>. Then next_token can return an option
        // and there can't be a misuse of it.
        //

        match self.advance() {
            Some(c) => {
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
                    x if x.is_lox_digit() => {
                        return self.make_number_token();
                    }
                    x if x.is_lox_alpha() => {
                        return self.make_identifier_token();
                    }
                    _ => {}
                }

                self.make_error_token("Unexpected character.")
            }
            None => Token {
                string: "",
                line: self.line,
                token_type: TokenType::TokenEof,
            },
        }
    }

    fn make_error_token(&self, string: &'a str) -> Token {
        Token {
            string: string,
            line: self.line,
            token_type: TokenType::TokenError,
        }
    }

    fn make_string_token(&mut self) -> Token {
        loop {
            match self.peek() {
                Some(c) => {
                    if c == '"' {
                        // The closing quote.
                        self.advance();
                        return self.make_token(TokenType::TokenString);
                    }

                    if c == '\n' {
                        self.line += 1;
                    }

                    self.advance();
                }

                None => {
                    // Didn't find a string terminator.
                    return self.make_error_token("Unterminated string.");
                }
            }
        }
    }

    fn make_number_token(&mut self) -> Token {
        // Consume the first full part of the number
        while self.peek().is_lox_digit() {
            self.advance();
        }

        // Look for a fractional part.
        if self.peek() == Some('.') && self.peek_next().is_lox_digit() {
            // Consume the ".".
            self.advance();

            while self.peek().is_lox_digit() {
                self.advance();
            }
        }

        self.make_token(TokenType::TokenNumber)
    }

    // Check if the identifier we matched is a keyword.
    fn check_keyword(mut iter: std::iter::Peekable<std::str::CharIndices<'a>>, rest: &str) -> bool {
        // First check length. If they don't match, can't possibly match.
        // TODO - how to check length without iterating through iterator, impossible??
        // Is it even better to do this than just checking char by char?
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
        while self.peek().is_lox_digit() || self.peek().is_lox_alpha() {
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
    use totems::assert_none;

    macro_rules! assert_eq_char {
        ($left:expr, $right:expr) => {
            assert_eq!($left, Some($right))
        };
    }

    #[test]
    fn peek_next_test() {
        let string = String::from("1.2");
        let mut scanner = Scanner::new(&string);

        assert_eq!(scanner.peek(), Some('1'));
        assert_eq!(scanner.peek_next(), Some('.'));

        assert_eq!(scanner.advance(), Some('1'));
        assert_eq!(scanner.peek(), Some('.'));
        assert_eq!(scanner.peek_next(), Some('2'));

        assert_eq_char!(scanner.advance(), '.');
        assert_eq_char!(scanner.advance(), '2');

        assert_none!(scanner.advance());
        assert_none!(scanner.advance());
        assert_none!(scanner.peek());
        assert_none!(scanner.peek_next());
    }

    #[test]
    fn skip_whitespace_test() {
        let string =
            String::from("    abc  \n\t   \n def  /qafe \n // ddd eeeee gasdfwe \t \t \n x");
        let mut scanner = Scanner::new(&string);

        // should skip up to abc
        assert_eq_char!(scanner.peek(), ' ');
        scanner.skip_whitespace();
        assert_eq_char!(scanner.advance(), 'a');
        assert_eq_char!(scanner.advance(), 'b');
        assert_eq_char!(scanner.advance(), 'c');

        // should skip up to def, with line incremented by two.
        scanner.skip_whitespace();
        assert_eq!(scanner.line, 3);
        assert_eq_char!(scanner.advance(), 'd');
        assert_eq_char!(scanner.advance(), 'e');
        assert_eq_char!(scanner.advance(), 'f');

        // skip up to /
        scanner.skip_whitespace();
        assert_eq_char!(scanner.advance(), '/');
        assert_eq_char!(scanner.advance(), 'q');
        assert_eq_char!(scanner.advance(), 'a');
        assert_eq_char!(scanner.advance(), 'f');
        assert_eq_char!(scanner.advance(), 'e');

        // skip to the last x
        scanner.skip_whitespace();
        assert_eq!(scanner.line, 5);
        assert_eq_char!(scanner.advance(), 'x');
        assert_none!(scanner.advance());

        let string = String::from("//a\n\n\n//\nx");
        scanner = Scanner::new(&string);

        // should skip all the way up to x.
        scanner.skip_whitespace();
        assert_eq!(scanner.line, 5);
        assert_eq_char!(scanner.advance(), 'x');
        assert_none!(scanner.advance());
    }

    // test digit trait

    // test parsing number tokens
}

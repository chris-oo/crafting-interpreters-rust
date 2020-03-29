// TODO - use non-peeking iterators?
// TODO - is there any way to not use the string? Shouldn't the scanner just be some iterator/adapter instead?
pub struct Scanner<'a> {
    source: &'a String,
    start: std::iter::Peekable<std::str::CharIndices<'a>>,
    current: std::iter::Peekable<std::str::CharIndices<'a>>,
    line: i32,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
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
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub struct Token<'a> {
    pub string: &'a str, // The slice that actually holds the string containing the token
    pub line: i32,
    pub token_type: TokenType,
}

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
    fn make_token(&mut self, token_type: TokenType) -> Token<'a> {
        match (self.start.peek(), self.current.peek()) {
            (Some(start), Some(current)) => Token {
                string: &self.source[start.0..current.0],
                line: self.line,
                token_type: token_type,
            },
            (Some(start), None) => Token {
                string: &self.source[start.0..],
                line: self.line,
                token_type: token_type,
            },
            _ => {
                // Parser error, should never happen. Start should always be valid.
                panic!("start was invalid when calling make_token");
            }
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
        // match self.current.next() {
        //     Some((_, x)) => Option::Some(x),
        //     None => None,
        // }
        self.current.next().map(|(_, x)| x)
    }

    // Consume a character if it matches the supplied one, returning true or false if it matched.
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
                    // Consume all characters up to then next newline, not including the
                    // newline.
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

    pub fn scan_token(&mut self) -> Token<'a> {
        self.skip_whitespace();

        self.start = self.current.clone();

        // Instead, scan token should return an iterator/option. Basically
        // the scanner should be an iterator that takes a string and output
        // is Iterator<Item = Token>. Then next_token can return an option
        // and there can't be a misuse of it.
        //

        match self.advance() {
            Some('(') => return self.make_token(TokenType::TokenLeftParen),
            Some(')') => return self.make_token(TokenType::TokenRightParen),
            Some('{') => return self.make_token(TokenType::TokenLeftBrace),
            Some('}') => return self.make_token(TokenType::TokenRightBrace),
            Some(';') => return self.make_token(TokenType::TokenSemicolon),
            Some(',') => return self.make_token(TokenType::TokenComma),
            Some('.') => return self.make_token(TokenType::TokenDot),
            Some('-') => return self.make_token(TokenType::TokenMinus),
            Some('+') => return self.make_token(TokenType::TokenPlus),
            Some('/') => return self.make_token(TokenType::TokenSlash),
            Some('*') => return self.make_token(TokenType::TokenStar),
            // TODO - Probably can macro these double character matches too.
            Some('!') => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenBangEqual);
                } else {
                    return self.make_token(TokenType::TokenBang);
                }
            }
            Some('=') => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenEqualEqual);
                } else {
                    return self.make_token(TokenType::TokenEqual);
                }
            }
            Some('<') => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenLessEqual);
                } else {
                    return self.make_token(TokenType::TokenLess);
                }
            }
            Some('>') => {
                if self.match_character('=') {
                    return self.make_token(TokenType::TokenGreaterEqual);
                } else {
                    return self.make_token(TokenType::TokenGreater);
                }
            }
            Some('"') => {
                return self.make_string_token();
            }
            Some(x) if x.is_lox_digit() => {
                return self.make_number_token();
            }
            Some(x) if x.is_lox_alpha() => {
                return self.make_identifier_token();
            }
            Some(_) => return self.make_error_token("Unexpected character."),
            None => Token {
                string: "",
                line: self.line,
                token_type: TokenType::TokenEof,
            },
        }
    }

    fn make_error_token(&self, string: &'a str) -> Token<'a> {
        Token {
            string: string,
            line: self.line,
            token_type: TokenType::TokenError,
        }
    }

    fn make_string_token(&mut self) -> Token<'a> {
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

    fn make_number_token(&mut self) -> Token<'a> {
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
    fn check_keyword(
        &mut self,
        mut iter: std::iter::Peekable<std::str::CharIndices<'a>>,
        rest: &str,
    ) -> bool {
        // Check each character if they match.
        for c in rest.chars() {
            match iter.next() {
                Some((_, x)) if x == c => {}
                _ => {
                    // Either length or character doesn't match.
                    return false;
                }
            }
        }

        // Length should match, so iter should be current.
        return iter.next().as_ref() == self.current.peek();
    }

    fn make_identifier_token(&mut self) -> Token<'a> {
        while self.peek().is_lox_digit() || self.peek().is_lox_alpha() {
            self.advance();
        }

        // Makes a keyword token if matches, otherwise returns an identifier token.
        // TODO - seems to me this could be a closure? or maybe not because of Token Type?
        macro_rules! make_keyword {
            ($iter:ident, $rest:expr, $token_type:tt) => {
                if self.check_keyword($iter, $rest) {
                    return self.make_token(TokenType::$token_type);
                } else {
                    return self.make_token(TokenType::TokenIdentifier);
                }
            };
        }

        // Produce a token with the proper type.
        let mut iter = self.start.clone();
        match iter.next() {
            Some((_, 'a')) => make_keyword!(iter, "nd", TokenAnd),
            Some((_, 'c')) => make_keyword!(iter, "lass", TokenClass),
            Some((_, 'e')) => make_keyword!(iter, "lse", TokenElse),
            Some((_, 'f')) => match iter.next() {
                Some((_, 'a')) => make_keyword!(iter, "lse", TokenFalse),
                Some((_, 'o')) => make_keyword!(iter, "r", TokenFor),
                Some((_, 'u')) => make_keyword!(iter, "n", TokenFun),
                Some(_) => (),
                None => {}
            },
            Some((_, 'i')) => make_keyword!(iter, "f", TokenIf),
            Some((_, 'n')) => make_keyword!(iter, "il", TokenNil),
            Some((_, 'o')) => make_keyword!(iter, "r", TokenOr),
            Some((_, 'p')) => make_keyword!(iter, "rint", TokenPrint),
            Some((_, 'r')) => make_keyword!(iter, "eturn", TokenReturn),
            Some((_, 's')) => make_keyword!(iter, "uper", TokenSuper),
            Some((_, 't')) => match iter.next() {
                Some((_, 'h')) => make_keyword!(iter, "is", TokenThis),
                Some((_, 'r')) => make_keyword!(iter, "ue", TokenTrue),
                Some(_) => (),
                None => (),
            },
            Some((_, 'v')) => make_keyword!(iter, "ar", TokenVar),
            Some((_, 'w')) => make_keyword!(iter, "hile", TokenWhile),
            Some(_) => (),
            None => panic!("Identifier parsing called on invalid start."),
        }

        self.make_token(TokenType::TokenIdentifier)
    }
}

#[cfg(test)]
mod tests {
    use super::Scanner;
    use super::Token;
    use super::TokenType;
    use totems::assert_none;

    macro_rules! assert_eq_char {
        ($left:expr, $right:expr) => {
            assert_eq!($left, Some($right))
        };
    }

    macro_rules! token {
        ($str:expr, $line:expr, $type:expr) => {
            Token {
                string: $str,
                line: $line,
                token_type: $type,
            }
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

    fn test_check_keyword() {}

    // test digit trait

    // test parsing number tokens
    #[test]
    fn simple_test() {
        let string = String::from("print 1 + a;");
        let mut scanner = Scanner::new(&string);

        assert_eq!(
            scanner.scan_token(),
            token!("print", 1, TokenType::TokenPrint)
        );
        assert_eq!(scanner.scan_token(), token!("1", 1, TokenType::TokenNumber));
        assert_eq!(scanner.scan_token(), token!("+", 1, TokenType::TokenPlus));
        assert_eq!(
            scanner.scan_token(),
            token!("a", 1, TokenType::TokenIdentifier)
        );
        assert_eq!(
            scanner.scan_token(),
            token!(";", 1, TokenType::TokenSemicolon)
        );
        assert_eq!(scanner.scan_token(), token!("", 1, TokenType::TokenEof));
    }

    // test parsing keywords
    fn keywords_test() {}
}

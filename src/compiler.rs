use crate::bytecode::Opcodes;
use crate::chunk::Chunk;
use crate::debug::DEBUG_PRINT_CODE;
use crate::lox_string_table::LoxStringTable;
use crate::scanner;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::value::Value;
use crate::vm::InterpretError;

use std::str::FromStr;

macro_rules! opcode_u8 {
    ($opcode:tt) => {
        Opcodes::$opcode as u8
    };
}

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    compiling_chunk: Chunk,
    string_table: &'a mut LoxStringTable,
}

#[derive(PartialOrd, PartialEq)]
enum Precedence {
    PrecNone = 0,
    PrecAssignment, // =
    PrecOr,         // or
    PrecAnd,        // and
    PrecEquality,   // == !=
    PrecComparison, // < > <= >=
    PrecTerm,       // + -
    PrecFactor,     // * /
    PrecUnary,      // ! -
    PrecCall,       // . ()
    PrecPrimary,
}

struct ParseRule<'p, 'a> {
    prefix: Option<fn(&'p mut Parser<'a>, bool)>,
    infix: Option<fn(&'p mut Parser<'a>, bool)>,
    precedence: Precedence,
}

impl Precedence {
    fn get_next_highest(&self) -> Self {
        // This is the equivalent in C as:
        // (Precedence)(rule->precedence + 1)
        // TODO - better way to do this?
        match self {
            Precedence::PrecNone => Precedence::PrecAssignment,
            Precedence::PrecAssignment => Precedence::PrecOr,
            Precedence::PrecOr => Precedence::PrecAnd,
            Precedence::PrecAnd => Precedence::PrecEquality,
            Precedence::PrecEquality => Precedence::PrecComparison,
            Precedence::PrecComparison => Precedence::PrecTerm,
            Precedence::PrecTerm => Precedence::PrecFactor,
            Precedence::PrecFactor => Precedence::PrecUnary,
            Precedence::PrecUnary => Precedence::PrecCall,
            Precedence::PrecCall => Precedence::PrecPrimary,
            Precedence::PrecPrimary => panic!("tried to upgrade highest"),
        }
    }
}

impl<'a> Parser<'a> {
    fn new(string_table: &'a mut LoxStringTable, source: &'a std::string::String) -> Self {
        Parser {
            scanner: scanner::Scanner::new(source),
            current: Token {
                string: "Parser placeholder.",
                line: -1,
                token_type: TokenType::TokenError,
            },
            previous: Token {
                string: "Parser placeholder.",
                line: -1,
                token_type: TokenType::TokenError,
            },
            had_error: false,
            panic_mode: false,
            compiling_chunk: Chunk::new(),
            string_table: string_table,
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        // TODO - this smells to me like scanner really should be an iterator and we range for over this.
        loop {
            self.current = self.scanner.scan_token();

            if self.current.token_type != TokenType::TokenError {
                break;
            }

            self.error_at_current(self.current.string);
        }
    }

    fn consume(&mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn check(&self, token_type: TokenType) -> bool {
        self.current.token_type == token_type
    }

    fn match_token(&mut self, token_type: TokenType) -> bool {
        if !self.check(token_type) {
            return false;
        }
        self.advance();
        return true;
    }

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PrecAssignment);
    }

    fn var_declaration(&mut self) {
        let global = self.parse_variable("Expect variable name.");

        if self.match_token(TokenType::TokenEqual) {
            self.expression();
        } else {
            self.emit_byte(opcode_u8!(OpNil));
        }
        self.consume(
            TokenType::TokenSemicolon,
            "Expect ';' after variable declaration",
        );

        self.define_variable(global);
    }

    fn expression_statement(&mut self) {
        self.expression();
        self.consume(TokenType::TokenSemicolon, "Expect ';' after value.");
        self.emit_byte(opcode_u8!(OpPop));
    }

    fn print_statement(&mut self) {
        self.expression();
        self.consume(TokenType::TokenSemicolon, "Expect ';' after value.");
        self.emit_byte(opcode_u8!(OpPrint));
    }

    fn synchronize(&mut self) {
        self.panic_mode = false;

        while self.current.token_type != TokenType::TokenEof {
            if self.previous.token_type == TokenType::TokenSemicolon {
                return;
            }

            match self.current.token_type {
                TokenType::TokenClass
                | TokenType::TokenFun
                | TokenType::TokenVar
                | TokenType::TokenFor
                | TokenType::TokenIf
                | TokenType::TokenWhile
                | TokenType::TokenPrint
                | TokenType::TokenReturn => {
                    return;
                }
                _ => {}
            }

            self.advance();
        }
    }

    fn declaration(&mut self) {
        if self.match_token(TokenType::TokenVar) {
            self.var_declaration();
        } else {
            self.statement();
        }

        if self.panic_mode {
            self.synchronize();
        }
    }

    fn statement(&mut self) {
        if self.match_token(TokenType::TokenPrint) {
            self.print_statement();
        } else {
            self.expression_statement();
        }
    }

    fn emit_byte(&mut self, byte: u8) {
        let line = self.previous.line;
        self.current_chunk().write_chunk(byte, line);
    }

    fn emit_bytes(&mut self, byte1: u8, byte2: u8) {
        self.emit_byte(byte1);
        self.emit_byte(byte2);
    }

    fn end_compiler(&mut self) {
        self.emit_return();

        if DEBUG_PRINT_CODE && !self.had_error {
            self.current_chunk().dissasemble("code");
        }
    }

    fn emit_return(&mut self) {
        self.emit_byte(Opcodes::OpReturn as u8);
    }

    fn current_chunk(&mut self) -> &mut Chunk {
        return &mut self.compiling_chunk;
    }

    fn error_at_current(&mut self, message: &str) {
        self.error_at(self.current, message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(self.previous, message);
    }

    // TODO - is it possible to borrow the token instead of copying?
    fn error_at(&mut self, token: Token, message: &str) {
        if self.panic_mode {
            return;
        }

        self.panic_mode = true;

        eprint!("[line {}] Error", token.line);

        match token.token_type {
            TokenType::TokenEof => {
                eprint!(" at end");
            }
            TokenType::TokenError => {}
            _ => {
                eprint!(" at '{}'", token.string);
            }
        }

        eprint!(": {}\n", message);
        self.had_error = true;
    }

    fn number(&mut self, can_assign: bool) {
        let value = Value::ValNumber(
            f64::from_str(self.previous.string).expect("number token contained not a number"),
        );
        self.emit_constant(value);
    }

    fn string(&mut self, can_assign: bool) {
        let str_slice = self.previous.string;
        // Skip leading and trailing '"' character
        let value = Value::ValObjString(
            self.string_table
                .allocate_string_from_str(&str_slice[1..str_slice.len() - 1]),
        );

        self.emit_constant(value);
    }

    fn named_variable(&mut self, token: Token, can_assign: bool) {
        let arg = self.identifier_constant(token);

        if can_assign && self.match_token(TokenType::TokenEqual) {
            self.expression();
            self.emit_bytes(opcode_u8!(OpSetGlobal), arg);
        } else {
            self.emit_bytes(opcode_u8!(OpGetGlobal), arg);
        }
    }

    fn variable(&mut self, can_assign: bool) {
        let token = self.previous;
        self.named_variable(token, can_assign);
    }

    fn emit_constant(&mut self, value: Value) {
        let constant = self.make_constant(value);
        self.emit_bytes(Opcodes::OpConstant as u8, constant);
    }

    fn make_constant(&mut self, value: Value) -> u8 {
        let constant = self.current_chunk().add_constant(value);

        if constant > std::u8::MAX as usize {
            self.error("Too many constants in one chunk.");
            return 0;
        }

        constant as u8
    }

    fn grouping(&mut self, can_assign: bool) {
        self.expression();
        self.consume(TokenType::TokenRightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self, can_assign: bool) {
        let operator_type = self.previous.token_type;

        // Compile the operand.
        self.parse_precedence(Precedence::PrecUnary);

        // Emit the operator instruction.
        match operator_type {
            TokenType::TokenBang => self.emit_byte(Opcodes::OpNot as u8),
            TokenType::TokenMinus => self.emit_byte(Opcodes::OpNegate as u8),
            _ => unreachable!(),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {
        self.advance();

        let prefix_rule_option = self.get_rule(self.previous.token_type).prefix;

        match prefix_rule_option {
            Some(prefix_rule) => {
                let can_assign = precedence <= Precedence::PrecAssignment;
                prefix_rule(self, can_assign);

                while precedence <= self.get_rule(self.current.token_type).precedence {
                    self.advance();
                    // TODO - c parser doesn't check null for infix. This would be a parser rules bug (aka table is wrong).
                    let infix_rule = self.get_rule(self.previous.token_type).infix.unwrap();
                    infix_rule(self, can_assign);
                }

                if can_assign && self.match_token(TokenType::TokenEqual) {
                    self.error("Invalid assignment target.");
                }
            }
            None => {
                self.error("Expect expression.");
            }
        }
    }

    fn identifier_constant(&mut self, name: Token) -> u8 {
        let obj = Value::ValObjString(self.string_table.allocate_string_from_str(name.string));
        self.make_constant(obj)
    }

    fn parse_variable(&mut self, error_message: &str) -> u8 {
        self.consume(TokenType::TokenIdentifier, error_message);
        let token = self.previous;
        self.identifier_constant(token)
    }

    fn define_variable(&mut self, global: u8) {
        self.emit_bytes(opcode_u8!(OpDefineGlobal), global);
    }

    fn binary(&mut self, can_assign: bool) {
        // Remember the operator.
        let operator_type = self.previous.token_type;

        // Compile the right operand.
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.get_next_highest());

        // Emit the operator instruction.
        match operator_type {
            TokenType::TokenBangEqual => self.emit_bytes(opcode_u8!(OpEqual), opcode_u8!(OpNot)),
            TokenType::TokenEqualEqual => self.emit_byte(opcode_u8!(OpEqual)),
            TokenType::TokenGreater => self.emit_byte(opcode_u8!(OpGreater)),
            TokenType::TokenGreaterEqual => self.emit_bytes(opcode_u8!(OpLess), opcode_u8!(OpNot)),
            TokenType::TokenLess => self.emit_byte(opcode_u8!(OpLess)),
            TokenType::TokenLessEqual => self.emit_bytes(opcode_u8!(OpGreater), opcode_u8!(OpNot)),
            TokenType::TokenPlus => self.emit_byte(opcode_u8!(OpAdd)),
            TokenType::TokenMinus => self.emit_byte(opcode_u8!(OpSubtract)),
            TokenType::TokenStar => self.emit_byte(opcode_u8!(OpMultiply)),
            TokenType::TokenSlash => self.emit_byte(opcode_u8!(OpDivide)),
            _ => unreachable!(),
        }
    }

    fn literal(&mut self, can_assign: bool) {
        match self.previous.token_type {
            TokenType::TokenFalse => self.emit_byte(opcode_u8!(OpFalse)),
            TokenType::TokenNil => self.emit_byte(opcode_u8!(OpNil)),
            TokenType::TokenTrue => self.emit_byte(opcode_u8!(OpTrue)),
            _ => unreachable!(),
        }
    }

    #[rustfmt::skip]
    fn get_rule<'p>(&self, token_type: TokenType) -> ParseRule<'p, 'a> {
        // NOTE: in C, this was done as a static table of function pointers.
        // I was wrong about that not working, it can work. However, it would
        // require casting the enum back to an integer (seems yuck).
        //
        // Replace it instead with a match expression.
        // TODO - does this performance wise match the C lookup table?

        macro_rules! make_rule {
            ($prefix:expr, $infix:expr, $precedence:tt) => {
                ParseRule {
                    prefix: $prefix,
                    infix: $infix,
                    precedence: Precedence::$precedence,
                }
            };
        }

        // This is the big table of rules.
        match token_type {
            TokenType::TokenLeftParen    => make_rule!(Some(Self::grouping),    None,               PrecNone),
            TokenType::TokenRightParen   => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenLeftBrace    => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenRightBrace   => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenComma        => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenDot          => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenMinus        => make_rule!(Some(Self::unary),       Some(Self::binary), PrecTerm),
            TokenType::TokenPlus         => make_rule!(None,                    Some(Self::binary), PrecTerm),
            TokenType::TokenSemicolon    => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenSlash        => make_rule!(None,                    Some(Self::binary), PrecFactor),
            TokenType::TokenStar         => make_rule!(None,                    Some(Self::binary), PrecFactor),
            TokenType::TokenBang         => make_rule!(Some(Self::unary),       None,               PrecNone),
            TokenType::TokenBangEqual    => make_rule!(None,                    Some(Self::binary), PrecEquality),
            TokenType::TokenEqual        => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenEqualEqual   => make_rule!(None,                    Some(Self::binary), PrecEquality),
            TokenType::TokenGreater      => make_rule!(None,                    Some(Self::binary), PrecComparison),
            TokenType::TokenGreaterEqual => make_rule!(None,                    Some(Self::binary), PrecComparison),
            TokenType::TokenLess         => make_rule!(None,                    Some(Self::binary), PrecComparison),
            TokenType::TokenLessEqual    => make_rule!(None,                    Some(Self::binary), PrecComparison),
            TokenType::TokenIdentifier   => make_rule!(Some(Self::variable),    None,               PrecNone),
            TokenType::TokenString       => make_rule!(Some(Self::string),      None,               PrecNone),
            TokenType::TokenNumber       => make_rule!(Some(Self::number),      None,               PrecNone),
            TokenType::TokenAnd          => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenClass        => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenElse         => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenFalse        => make_rule!(Some(Self::literal),     None,               PrecNone),
            TokenType::TokenFor          => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenFun          => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenIf           => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenNil          => make_rule!(Some(Self::literal),     None,               PrecNone),
            TokenType::TokenOr           => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenPrint        => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenReturn       => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenSuper        => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenThis         => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenTrue         => make_rule!(Some(Self::literal),     None,               PrecNone),
            TokenType::TokenVar          => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenWhile        => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenError        => make_rule!(None,                    None,               PrecNone),
            TokenType::TokenEof          => make_rule!(None,                    None,               PrecNone),
        }
    }
}

pub fn compile(
    string_table: &mut LoxStringTable,
    source: &String,
) -> Result<Chunk, InterpretError> {
    let mut parser = Parser::new(string_table, source);

    parser.advance();

    while !parser.match_token(TokenType::TokenEof) {
        parser.declaration();
    }

    parser.end_compiler();

    if parser.had_error {
        return Err(InterpretError::InterpretCompileError);
    }

    Ok(parser.compiling_chunk)
}

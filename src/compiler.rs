use crate::bytecode::Opcodes;
use crate::bytecode::Value;
use crate::chunk::Chunk;
use crate::scanner;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::vm::InterpretResult;

use std::str::FromStr;

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    compiling_chunk: Chunk,
}

enum Precedence {
    PrecNone,
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

struct ParseRule {
    prefix: Option<fn(&mut Parser)>,
    infix: Option<fn(&mut Parser)>,
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
    fn new(source: &'a std::string::String) -> Self {
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
        }
    }

    fn advance(&mut self) {
        self.previous = self.current.clone();

        // TODO - this smells to me like scanner really should be an iterator and we range for over this.
        loop {
            self.current = self.scanner.scan_token();

            if self.current.token_type == TokenType::TokenError {
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

    fn expression(&mut self) {
        self.parse_precedence(Precedence::PrecAssignment);
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
                eprint!("at end");
            }
            TokenType::TokenError => {}
            _ => {
                eprint!(" at '{}'", token.string);
            }
        }

        eprint!(": {}\n", message);
        self.had_error = true;
    }

    fn number(&mut self) {
        let value =
            Value::from_str(self.previous.string).expect("number token contained not a number");
        self.emit_constant(value);
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

    fn grouping(&mut self) {
        self.expression();
        self.consume(TokenType::TokenRightParen, "Expect ')' after expression.");
    }

    fn unary(&mut self) {
        let operator_type = self.previous.token_type;

        // Compile the operand.
        self.parse_precedence(Precedence::PrecUnary);

        // Emit the operator instruction.
        match operator_type {
            TokenType::TokenMinus => self.emit_byte(Opcodes::OpNegate as u8),
            _ => panic!("Unreachable"),
        }
    }

    fn parse_precedence(&mut self, precedence: Precedence) {}

    fn binary(&mut self) {
        // Remember the operator.
        let operator_type = self.previous.token_type;

        // Compile the right operand.
        let rule = self.get_rule(operator_type);
        self.parse_precedence(rule.precedence.get_next_highest());

        // Emit the operator instruction.
        match operator_type {
            TokenType::TokenPlus => self.emit_byte(Opcodes::OpAdd as u8),
            TokenType::TokenMinus => self.emit_byte(Opcodes::OpSubtract as u8),
            TokenType::TokenStar => self.emit_byte(Opcodes::OpMultiply as u8),
            TokenType::TokenSlash => self.emit_byte(Opcodes::OpDivide as u8),
            _ => panic!("Unreachable"),
        }
    }

    fn get_rule(&mut self, token_type: TokenType) -> ParseRule {
        // NOTE: in C, this was done as a static table of function pointers.
        // That doesn't work, because having function pointers to member
        // functions isn't really possible, and is also a really bad design.
        //
        // Replace it instead with a match expression.
        // TODO - does this performance wise match the C lookup table?
        match token_type {
            TokenType::TokenLeftParen => ParseRule {
                prefix: Some(Self::grouping),
                infix: None,
                precedence: Precedence::PrecNone,
            },
            _ => unimplemented!(),
        }
    }
}

pub fn compile(source: &String) -> Result<Chunk, InterpretResult> {
    let mut parser = Parser::new(source);

    parser.advance();
    parser.expression();
    parser.consume(TokenType::TokenEof, "Expect end of expression");
    parser.end_compiler();

    if parser.had_error {
        return Err(InterpretResult::InterpretCompileError);
    }

    Ok(parser.compiling_chunk)
}

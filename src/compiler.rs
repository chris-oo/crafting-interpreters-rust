use crate::bytecode::Opcodes;
use crate::chunk::Chunk;
use crate::scanner;
use crate::scanner::Scanner;
use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::vm::InterpretResult;

struct Parser<'a> {
    scanner: Scanner<'a>,
    current: Token<'a>,
    previous: Token<'a>,
    had_error: bool,
    panic_mode: bool,
    compiling_chunk: Chunk,
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

    fn consume(&'a mut self, token_type: TokenType, message: &str) {
        if self.current.token_type == token_type {
            self.advance();
            return;
        }

        self.error_at_current(message);
    }

    fn expression(&mut self) {}

    fn emit_byte(&mut self, byte: u8) {
        self.current_chunk().write_chunk(byte, self.previous.line);
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
        self.error_at(&self.current, message);
    }

    fn error(&mut self, message: &str) {
        self.error_at(&self.previous, message);
    }

    fn error_at(&mut self, token: &Token, message: &str) {
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

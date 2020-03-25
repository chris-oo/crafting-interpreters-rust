use crate::bytecode::Opcodes;
use crate::bytecode::Value;
use crate::chunk::Chunk;

static DEBUG_TRACE_EXECUTION: bool = true;

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

pub struct VM {
    chunk: Option<Chunk>,
    ip: usize,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Option::None,
            ip: 0,
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        // TODO - how does one create an option pointer from a ref? is it not
        // possible in safe rust because the lifetime of the called object
        // cannot be determined?
        //
        // What would be the idiomatic thing to do here, take an Option<Chunk> and take ownership of it?
        //
        // Probably take a reference counted pointer?
        self.chunk = Option::Some(chunk.clone());
        self.ip = 0;
        self.run()
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.as_ref().unwrap().code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte();
        self.chunk.as_ref().unwrap().constants[index as usize]
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if DEBUG_TRACE_EXECUTION {
                self.chunk
                    .as_ref()
                    .unwrap()
                    .dissasemble_instruction(self.ip);
            }

            let instruction = num::FromPrimitive::from_u8(self.read_byte());

            match instruction {
                Some(Opcodes::OpReturn) => return InterpretResult::InterpretOk,
                Some(Opcodes::OpConstant) => {
                    let constant = self.read_constant();
                    println!("DEBUG: constant {:?}", constant);
                }
                // Some(_) => unimplemented!("Opcode not implemented {}", self.code[offset]),
                None => return InterpretResult::InterpretRuntimeError,
            }
        }
    }
}

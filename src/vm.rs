use crate::bytecode::Opcodes;
use crate::bytecode::Value;
use crate::chunk::Chunk;

const DEBUG_TRACE_EXECUTION: bool = true;

const STACK_MAX: usize = 256;

pub enum InterpretResult {
    InterpretOk,
    InterpretCompileError,
    InterpretRuntimeError,
}

pub struct VM<'a> {
    chunk: Option<&'a Chunk>,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

impl<'a> VM<'a> {
    pub fn new() -> Self {
        VM {
            chunk: Option::None,
            ip: 0,
            stack: [0.0; STACK_MAX],
            stack_top: 0,
        }
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    pub fn interpret<'b: 'a>(&'a mut self, chunk: &'b Chunk) -> InterpretResult {
        // TODO - is this really the best way to do it?
        //
        // Maybe take a reference counted pointer?
        //
        // It probably makes more sense for the VM to "own" the whole chunklist once the
        // compilation phase is done.
        self.chunk = Option::Some(chunk);
        self.ip = 0;
        self.run()
    }

    fn push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top]
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
        macro_rules! binary_op {
            ($op:tt) => {
                let b = self.pop();
                let a = self.pop();
                self.push(a $op b);
            };
        }

        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                let mut i = 0;
                while i < self.stack_top {
                    print!("[{:?}]", self.stack[i]);
                    i += 1;
                }
                print!("\n");

                self.chunk
                    .as_ref()
                    .unwrap()
                    .dissasemble_instruction(self.ip);
            }

            let instruction = num::FromPrimitive::from_u8(self.read_byte());

            match instruction {
                Some(Opcodes::OpReturn) => {
                    println!("{:?}", self.pop());
                    return InterpretResult::InterpretOk;
                }

                Some(Opcodes::OpConstant) => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Some(Opcodes::OpNegate) => {
                    let value = -self.pop();
                    self.push(value);
                }
                Some(Opcodes::OpAdd) => {
                    binary_op!(+);
                }
                Some(Opcodes::OpSubtract) => {
                    binary_op!(-);
                }
                Some(Opcodes::OpMultiply) => {
                    binary_op!(*);
                }
                Some(Opcodes::OpDivide) => {
                    binary_op!(/);
                }
                // Some(_) => unimplemented!("Opcode not implemented"),
                None => return InterpretResult::InterpretRuntimeError,
            }
        }
    }
}

use crate::bytecode::Opcodes;
use crate::bytecode::Value;

#[derive(Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<i32>,
    pub constants: Vec<Value>,
}

impl Chunk {
    pub fn write_chunk(&mut self, byte: u8, line: i32) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_instruction(&mut self, instruction: Opcodes, line: i32) {
        self.write_chunk(instruction as u8, line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn reset(&mut self) {
        self.code.clear();
        self.lines.clear();
        self.constants.clear();
    }

    pub fn dissasemble(&self, name: &str) {
        println!("== {0} ==", name);
        let mut offset = 0;

        while offset < self.code.len() {
            offset += self.dissasemble_instruction(offset) as usize;
        }
    }

    pub fn new() -> Self {
        let chunk = Chunk {
            code: Vec::new(),
            lines: Vec::new(),
            constants: Vec::new(),
        };

        chunk
    }

    /// Disassembles the instruction starting at the specified offset.
    ///
    /// Returns the size of the disassembled instruction.
    pub fn dissasemble_instruction(&self, offset: usize) -> u8 {
        print!("{:04} ", offset);

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction = num::FromPrimitive::from_u8(self.code[offset]);

        match instruction {
            Some(Opcodes::OpReturn) => Chunk::simple_instruction("OP_RETURN", offset),
            Some(Opcodes::OpConstant) => Chunk::constant_instruction("OP_CONSTANT", self, offset),
            // Some(_) => unimplemented!("Opcode not implemented {}", self.code[offset]),
            None => {
                print!("Unknown opcode {0}\n", self.code[offset]);
                1
            }
        }
    }

    fn simple_instruction(name: &str, offset: usize) -> u8 {
        println!("{}\n", name);
        1
    }

    fn constant_instruction(name: &str, chunk: &Chunk, offset: usize) -> u8 {
        let constant = chunk.code[offset + 1];
        print!(
            "{:16} {:04} '{}'\n",
            name, constant, chunk.constants[constant as usize]
        );
        2
    }
}

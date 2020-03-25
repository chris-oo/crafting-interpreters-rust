extern crate num;
#[macro_use]
extern crate num_derive;

mod bytecode;
mod chunk;
mod vm;

fn main() {
    let mut vm = vm::VM::new();
    let mut chunk = chunk::Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.add_instruction(bytecode::Opcodes::OpConstant, 123);
    chunk.write_chunk(constant as u8, 123);
    chunk.add_instruction(bytecode::Opcodes::OpNegate, 123);

    chunk.add_instruction(bytecode::Opcodes::OpReturn, 123);
    chunk.dissasemble("test chunk");
    vm.interpret(&chunk);

    chunk.reset();
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_test() {
        assert_eq!(2 + 2, 4);
    }
}

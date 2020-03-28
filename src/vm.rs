use crate::bytecode::Opcodes;
use crate::bytecode::Value;
use crate::bytecode::DEBUG_TRACE_EXECUTION;
use crate::chunk::Chunk;
use crate::compiler;

const STACK_MAX: usize = 256;

pub enum InterpretResult {
    InterpretCompileError,
    InterpretRuntimeError,
}

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: [Value; STACK_MAX],
    stack_top: usize,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: [Value::ValNil; STACK_MAX],
            stack_top: 0,
        }
    }

    fn reset_stack(&mut self) {
        self.stack_top = 0;
    }

    // C took a format string, but rust you can call format!() instead. This
    // means the C style runtime_error function should be a macro instead?
    fn runtime_error_formatted(&mut self, message: &str) {
        eprintln!("{}", message);

        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line);

        self.reset_stack();
    }

    pub fn interpret(&mut self, source: &String) -> Result<(), InterpretResult> {
        self.chunk = compiler::compile(source)?;
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

    fn peek(&self, distance: usize) -> Value {
        self.stack[self.stack_top - 1 - distance]
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte();
        self.chunk.constants[index as usize]
    }

    fn run(&mut self) -> Result<(), InterpretResult> {
        macro_rules! binary_op {
            ($value_type:tt, $op:tt) => {
                match (self.peek(0), self.peek(1)) {
                    (Value::ValNumber(b), Value::ValNumber(a)) => {
                        // The match arm got the operand values, so just pop twice.
                        self.pop();
                        self.pop();
                        self.push(Value::$value_type(a $op b));
                    }
                    _ => {
                        self.runtime_error_formatted("Operands must be numbers.");
                        return Err(InterpretResult::InterpretRuntimeError);
                    }
                }
            };
        }

        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                let mut i = 0;
                while i < self.stack_top {
                    print!("[{}]", self.stack[i]);
                    i += 1;
                }
                print!("\n");

                self.chunk.dissasemble_instruction(self.ip);
            }

            let instruction = num::FromPrimitive::from_u8(self.read_byte());

            match instruction {
                Some(Opcodes::OpReturn) => {
                    println!("{}", self.pop());
                    return Ok(());
                }

                Some(Opcodes::OpConstant) => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Some(Opcodes::OpNegate) => match self.peek(0) {
                    Value::ValNumber(x) => {
                        self.pop();
                        self.push(Value::ValNumber(-x));
                    }
                    _ => {
                        self.runtime_error_formatted("Operand must be a number.");
                        return Err(InterpretResult::InterpretRuntimeError);
                    }
                },
                Some(Opcodes::OpNil) => {
                    self.push(Value::ValNil);
                }
                Some(Opcodes::OpTrue) => {
                    self.push(Value::ValBool(true));
                }
                Some(Opcodes::OpFalse) => {
                    self.push(Value::ValBool(false));
                }
                Some(Opcodes::OpEqual) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::ValBool(a == b));
                }
                Some(Opcodes::OpGreater) => binary_op!(ValBool, >),
                Some(Opcodes::OpLess) => binary_op!(ValBool, <),
                Some(Opcodes::OpAdd) => binary_op!(ValNumber, +),
                Some(Opcodes::OpSubtract) => binary_op!(ValNumber, -),
                Some(Opcodes::OpMultiply) => binary_op!(ValNumber, *),
                Some(Opcodes::OpDivide) => binary_op!(ValNumber, /),
                Some(Opcodes::OpNot) => {
                    let value = Value::ValBool(self.pop().is_falsey());
                    self.push(value);
                }
                // Some(_) => unimplemented!("Opcode not implemented"),
                None => return Err(InterpretResult::InterpretRuntimeError),
            }
        }
    }
}

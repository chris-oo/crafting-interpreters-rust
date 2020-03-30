use crate::bytecode::Opcodes;
use crate::chunk::Chunk;
use crate::compiler;
use crate::debug::DEBUG_TRACE_EXECUTION;
use crate::lox_string_table::LoxString;
use crate::lox_string_table::LoxStringTable;
use crate::value::Value;
use std::collections::HashMap;

// TODO - split compiler and vm runtime errors
pub enum InterpretError {
    InterpretCompileError,
    InterpretRuntimeError,
}

pub struct VM {
    chunk: Chunk,
    ip: usize,
    stack: Vec<Value>,
    globals: HashMap<LoxString, Value>,
    string_table: LoxStringTable,
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            stack: Vec::new(),
            globals: HashMap::new(),
            string_table: LoxStringTable::new(),
        }
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    // C took a format string, but rust you can call format!() instead. This
    // means the C style runtime_error function should be a macro instead?
    fn runtime_error_formatted(&mut self, message: &str) {
        eprintln!("{}", message);

        let line = self.chunk.lines[self.ip - 1];
        eprintln!("[line {}] in script", line);

        self.reset_stack();
    }

    pub fn interpret(&mut self, source: &String) -> Result<(), InterpretError> {
        self.chunk = compiler::compile(&mut self.string_table, source)?;
        self.ip = 0;

        self.run()
    }

    fn push(&mut self, value: Value) {
        // TODO - enforce some stack limit
        self.stack.push(value);
    }

    fn pop(&mut self) -> Value {
        // TODO - return result type w/ stack error instead of unwrap
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance: usize) -> &Value {
        // TODO - return result type w/ stack error instead of panic
        &self.stack[self.stack.len() - 1 - distance]
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.chunk.code[self.ip];
        self.ip += 1;
        byte
    }

    fn read_constant(&mut self) -> Value {
        let index = self.read_byte();
        // TODO - ick cloning constants to push onto the stack. Fine for small
        // things, terrible for strings.
        //
        // It would be better to have the stack have non-owning references. But
        // then who owns the objects? will need to be solved for GC
        //
        // Do we do what clox does and have the VM have a list of objects?
        self.chunk.constants[index as usize].clone()
    }

    fn read_string(&mut self) -> Result<LoxString, InterpretError> {
        match self.read_constant() {
            Value::ValObjString(string) => {
                return Ok(string);
            }
            _ => {
                self.runtime_error_formatted("OpDefineGlobal constant wasn't a string.");
                return Err(InterpretError::InterpretRuntimeError);
            }
        }
    }

    fn run(&mut self) -> Result<(), InterpretError> {
        macro_rules! binary_op {
            ($value_type:tt, $op:tt) => {
                // NOTE - This is different than clox. In clox we peek twice instead
                // of popping, for GC tracing.
                //
                // There's no GC yet, but this rust implementation (shouldn't?) let
                // things get freed until destructors are called, and since we keep
                // the variable alive in the match arm, we should be okay.
                match (self.pop(), self.pop()) {
                    (Value::ValNumber(b), Value::ValNumber(a)) => {
                        self.push(Value::$value_type(a $op b));
                    }
                    _ => {
                        self.runtime_error_formatted("Operands must be numbers.");
                        return Err(InterpretError::InterpretRuntimeError);
                    }
                }
            };
        }

        loop {
            if DEBUG_TRACE_EXECUTION {
                print!("          ");
                for value in &self.stack {
                    print!("[{}]", value);
                }
                print!("\n");

                self.chunk.dissasemble_instruction(self.ip);
            }

            let instruction = num::FromPrimitive::from_u8(self.read_byte());

            match instruction {
                Some(Opcodes::OpPrint) => {
                    println!("{}", self.pop());
                }

                Some(Opcodes::OpReturn) => {
                    return Ok(());
                }

                Some(Opcodes::OpConstant) => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                Some(Opcodes::OpNegate) => match self.peek(0).clone() {
                    Value::ValNumber(x) => {
                        self.pop();
                        self.push(Value::ValNumber(-x));
                    }
                    _ => {
                        self.runtime_error_formatted("Operand must be a number.");
                        return Err(InterpretError::InterpretRuntimeError);
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
                Some(Opcodes::OpPop) => {
                    self.pop();
                }
                Some(Opcodes::OpGetGlobal) => {
                    let name = self.read_string()?;
                    let global = self.globals.get(&name);

                    if global == None {
                        self.runtime_error_formatted(
                            format!("Undefined variable '{}'.", name.as_str()).as_str(),
                        );
                        return Err(InterpretError::InterpretRuntimeError);
                    }

                    let value = global.unwrap().clone();
                    self.push(value);
                }
                Some(Opcodes::OpDefineGlobal) => {
                    let name = self.read_string()?;
                    let value = self.peek(0).clone();
                    self.globals.insert(name, value);
                    self.pop();
                }
                Some(Opcodes::OpEqual) => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::ValBool(a == b));
                }
                Some(Opcodes::OpGreater) => binary_op!(ValBool, >),
                Some(Opcodes::OpLess) => binary_op!(ValBool, <),
                Some(Opcodes::OpAdd) => match (self.peek(0).clone(), self.peek(1).clone()) {
                    (Value::ValObjString(b), Value::ValObjString(a)) => {
                        self.pop();
                        self.pop();
                        let string = Value::ValObjString(self.string_table.concatenate(&a, &b));
                        self.push(string);
                    }
                    (Value::ValNumber(b), Value::ValNumber(a)) => {
                        self.pop();
                        self.pop();
                        self.push(Value::ValNumber(b + a));
                    }
                    _ => {
                        self.runtime_error_formatted("Operand must be two numbers or two strings.");
                        return Err(InterpretError::InterpretRuntimeError);
                    }
                },
                Some(Opcodes::OpSubtract) => binary_op!(ValNumber, -),
                Some(Opcodes::OpMultiply) => binary_op!(ValNumber, *),
                Some(Opcodes::OpDivide) => binary_op!(ValNumber, /),
                Some(Opcodes::OpNot) => {
                    let value = Value::ValBool(self.pop().is_falsey());
                    self.push(value);
                }
                // Some(_) => unimplemented!("Opcode not implemented"),
                None => return Err(InterpretError::InterpretRuntimeError),
            }
        }
    }
}

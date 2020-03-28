use std::fmt;

// TODO belongs in value.rs
#[derive(Clone, Copy, Debug)]
pub enum Value {
    ValBool(bool),
    ValNil,
    ValNumber(f64),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::ValBool(x) => write!(f, "{}", x),
            Value::ValNil => write!(f, "nil"),
            Value::ValNumber(x) => write!(f, "{}", x),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::ValBool(x) => !x,
            Value::ValNil => true,
            Value::ValNumber(_) => false,
        }
    }
}

#[derive(FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Opcodes {
    OpReturn,
    OpConstant,
    OpNil,
    OpTrue,
    OpFalse,
    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNot,
}

// TODO probably belongs in debug.rs
pub const DEBUG_PRINT_CODE: bool = true;
pub const DEBUG_TRACE_EXECUTION: bool = true;

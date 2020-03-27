pub type Value = f64;

#[derive(FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Opcodes {
    OpReturn,
    OpConstant,
    OpNegate,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
}

// TODO probably belongs in debug.rs
pub const DEBUG_PRINT_CODE: bool = true;
pub const DEBUG_TRACE_EXECUTION: bool = true;

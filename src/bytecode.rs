#[derive(FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Opcodes {
    OpReturn,
    OpConstant,
    OpNil,
    OpTrue,
    OpFalse,
    OpEqual,
    OpGreater,
    OpLess,
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

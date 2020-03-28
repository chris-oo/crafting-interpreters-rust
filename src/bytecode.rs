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

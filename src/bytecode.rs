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

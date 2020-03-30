#[derive(FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum Opcodes {
    OpReturn,
    OpPrint,
    OpConstant,
    OpNil,
    OpTrue,
    OpFalse,
    OpPop,
    OpGetGlobal,
    OpDefineGlobal,
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

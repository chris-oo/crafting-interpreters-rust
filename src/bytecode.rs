pub type Value = f64;

#[derive(FromPrimitive)]
pub enum Opcodes {
    OpReturn,
    OpConstant,
    OpNegate,
}

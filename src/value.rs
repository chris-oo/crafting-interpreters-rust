use crate::lox_string_table::LoxString;
use std::fmt;

// TODO - the book has two different nested types - values and objects.
// It seems to me they could be flattened? Does it make sense to do that?
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    ValBool(bool),
    ValNil,
    ValNumber(f64),
    ValObjString(LoxString),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::ValBool(x) => write!(f, "{}", x),
            Value::ValNil => write!(f, "nil"),
            Value::ValNumber(x) => write!(f, "{}", x),
            Value::ValObjString(x) => write!(f, "{:?}", x),
        }
    }
}

impl Value {
    pub fn is_falsey(&self) -> bool {
        match self {
            Value::ValBool(x) => !x,
            Value::ValNil => true,
            _ => false,
        }
    }
}

use std::fmt;

// TODO belongs in value.rs
#[derive(Clone, Copy, Debug, PartialEq)]
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

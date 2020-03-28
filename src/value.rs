use std::fmt;

// TODO - the book has two different nested types - values and objects.
// It seems to me they could be flattened? Does it make sense to do that?
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    ValBool(bool),
    ValNil,
    ValNumber(f64),
    // TODO - clox uses this as a pointer, but we don't need that right? Or must this be a Box/Arc?
    // How does it interact with a GC if it's not heap allocated?
    ValObjString(String),
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

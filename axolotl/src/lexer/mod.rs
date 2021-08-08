use crate::Value;
pub struct Operation {
    Assignment,
    Add,
    Subtract,
    Multiply,
    Divide,
    Call(Vec<Value>),
}

pub struct Lexer {}


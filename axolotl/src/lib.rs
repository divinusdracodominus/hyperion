use std::collections::HashMap;
pub mod lexer;

lazy_static! {
    static ref Result = {

    };
}

#[derive(Debug, Clone)]
pub enum Value {
    Str(String),
    Int(i32),
    Float(f32),
    LargeFloat(f64),
    LargeInt(i64),
    Map(HashMap<String, Value>),
    Array(Vec<Value>),
    Flux(Flux),
}

/// this acts as a form of enum
#[derive(Debug, Clone)]
pub struct Flux {
    varient: String,
    value: Box<Value>,
    possibilities: Vec<String>,
}

impl Flux {
    pub fn new(varient: String, value: Value, possibilities: Vec<String>) -> Flux {
        
    }
}
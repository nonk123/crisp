use crate::parsers::{determine_parser, ParserError};

use std::collections::HashMap;

#[derive(Debug, Clone, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn new(string: &String) -> Self {
        Self(string.to_string())
    }

    pub fn from_str(string: &str) -> Self {
        Self(string.to_string())
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

pub type CrispInteger = i32;
pub type SymbolTable = HashMap<Symbol, Value>;

#[derive(Debug, Clone)]
pub enum Value {
    Void,
    Integer(CrispInteger),
    Symbol { symbol: Symbol, quoted: bool },
}

impl Value {
    pub fn eval(&self, environment: &Environment) -> Value {
        match self {
            Value::Symbol { symbol, quoted } => {
                if *quoted {
                    self.clone()
                } else {
                    environment.lookup(&symbol).unwrap_or(Value::Void)
                }
            }
            _ => self.clone(),
        }
    }
}

#[derive(Debug)]
pub enum EvalError {
    ErrorDuringParsing(ParserError),
    NoParserAvailable,
}

#[derive(Debug, Clone)]
pub struct Closure(SymbolTable);

impl Closure {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn put(&mut self, key: Symbol, value: Value) {
        self.0.insert(key, value);
    }

    pub fn put_str(&mut self, key: &str, value: Value) {
        self.put(Symbol::from_str(key), value);
    }
}

pub struct Environment {
    stack: Vec<Closure>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: vec![Closure::new()],
        }
    }

    pub fn top_level(&mut self) -> &mut Closure {
        self.stack.first_mut().unwrap()
    }

    pub fn push(&mut self, closure: Closure) {
        self.stack.push(closure);
    }

    pub fn lookup(&self, symbol: &Symbol) -> Option<Value> {
        let mut stack = self.stack.clone();

        loop {
            match stack.pop() {
                Some(closure) => {
                    if let Some(value) = closure.0.get(symbol) {
                        return Some(value.clone());
                    }
                }
                None => return None,
            }
        }
    }
}

pub type EvalResult = Result<Value, EvalError>;

pub fn eval(environment: &Environment, buffer: String) -> EvalResult {
    if let Some(parser) = determine_parser(&buffer) {
        match parser.parse(&buffer) {
            Ok(value) => Ok(value.eval(&environment)),
            Err(err) => Err(EvalError::ErrorDuringParsing(err)),
        }
    } else {
        Err(EvalError::NoParserAvailable)
    }
}

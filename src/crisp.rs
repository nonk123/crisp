use crate::parsers::{parse, ParserError};

use std::collections::HashMap;

#[derive(Debug, Clone, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn from_str(string: &str) -> Self {
        Self(string.to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

pub type Integer = i32;
pub type Function = fn(&mut Environment, Vec<Value>) -> EvalResult;

pub type SymbolTable = HashMap<Symbol, Value>;

#[derive(Debug)]
pub enum EvalError {
    ArgsMismatch,
    VariableIsVoid(String),
    FunctionDefinitionIsVoid(String),
    FailedToParse(ParserError),
}

pub type EvalResult = Result<Value, EvalError>;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    T,
    Integer(Integer),
    Symbol { symbol: Symbol, quoted: bool },
    Funcall(Symbol, Vec<Value>),
    List(Vec<Value>),
}

impl Value {
    pub fn eval(&self, environment: &mut Environment) -> EvalResult {
        match self {
            Value::Symbol { symbol, quoted } => {
                if *quoted {
                    Ok(self.clone())
                } else {
                    match environment.lookup(&symbol) {
                        Some(value) => Ok(value),
                        None => Err(EvalError::VariableIsVoid(symbol.to_string())),
                    }
                }
            }
            Value::Funcall(symbol, args) => environment.call(symbol, args.to_vec()),
            Value::List(elements) => {
                let mut evaluated: Vec<Value> = Vec::new();

                for element in elements.iter() {
                    evaluated.push(element.eval(environment)?);
                }

                Ok(Value::List(evaluated))
            }
            _ => Ok(self.to_owned()),
        }
    }
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
    functions_table: HashMap<Symbol, Function>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: vec![Closure::new()],
            functions_table: HashMap::new(),
        }
    }

    pub fn top_level(&mut self) -> &mut Closure {
        self.stack.first_mut().unwrap()
    }

    pub fn current(&mut self) -> &mut Closure {
        self.stack.last_mut().unwrap()
    }

    pub fn push_to_stack(&mut self, closure: Closure) {
        self.stack.push(closure);
    }

    pub fn push_new(&mut self) {
        self.push_to_stack(Closure::new())
    }

    pub fn pop(&mut self) -> Option<Closure> {
        self.stack.pop()
    }

    pub fn add_function(&mut self, key: Symbol, function: Function) {
        self.functions_table.insert(key, function);
    }

    pub fn add_function_str(&mut self, key: &str, function: Function) {
        self.add_function(Symbol::from_str(key), function);
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

    pub fn function_lookup(&self, symbol: &Symbol) -> Option<Function> {
        self.functions_table.get(symbol).cloned()
    }

    pub fn call(&mut self, symbol: &Symbol, args: Vec<Value>) -> EvalResult {
        if let Some(function) = self.function_lookup(symbol) {
            self.push_new();
            let value = function(self, args);
            self.pop();

            value
        } else {
            Err(EvalError::FunctionDefinitionIsVoid(symbol.to_string()))
        }
    }
}

pub fn eval(environment: &mut Environment, buffer: String) -> EvalResult {
    parse(&buffer)
        .map_err(EvalError::FailedToParse)?
        .eval(environment)
}

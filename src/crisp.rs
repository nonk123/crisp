use crate::parsers::{parse, ParserError};

use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Clone, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub quote: Quote,
    pub rest: bool,
}

impl Symbol {
    pub fn new(name: String, quote: Quote, rest: bool) -> Self {
        Self { name, quote, rest }
    }

    pub fn from_str(name: &str) -> Self {
        Self::new(name.to_string(), Quote::None, false)
    }

    // Used in `tests`.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }

    pub fn to_string(&self) -> String {
        self.name.to_string()
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.quote == other.quote && self.rest == other.rest
    }
}

pub type Integer = i32;

type Builtin = fn(&mut Environment, Vec<Value>) -> EvalResult;

#[derive(Clone)]
pub struct Defun {
    body: Value,
    takes: Vec<Symbol>,
}

#[derive(Clone)]
pub enum Function {
    Builtin(Builtin),
    Defun(Defun),
}

impl Function {
    pub fn new_defun(body: Value, takes: Vec<Symbol>) -> Self {
        Self::Defun(Defun { body, takes })
    }

    pub fn new_builtin(function: Builtin) -> Self {
        Self::Builtin(function)
    }

    fn eval_defun(
        &self,
        environment: &mut Environment,
        defun: &Defun,
        mut args: Vec<Value>,
    ) -> EvalResult {
        for symbol in defun.takes.iter() {
            if symbol.rest {
                let value = {
                    let list = Value::List(args);

                    match symbol.quote {
                        Quote::Single => list,
                        _ => list.eval(environment)?,
                    }
                };

                environment.current().put(symbol.clone(), value);

                break;
            } else {
                let value = {
                    let arg = match args.get(0) {
                        Some(_) => args.remove(0),
                        None => {
                            return Err(EvalError::ArgsMismatch(
                                "Not enough args passed to the function".into(),
                            ))
                        }
                    };

                    match symbol.quote {
                        Quote::Single => arg.clone(),
                        _ => arg.eval(environment)?,
                    }
                };

                environment.current().put(symbol.clone(), value);
            }
        }

        let result = defun.body.eval(environment);

        result
    }

    pub fn call(&self, environment: &mut Environment, args: Vec<Value>) -> EvalResult {
        match self {
            Self::Builtin(function) => function(environment, args),
            Self::Defun(defun) => self.eval_defun(environment, defun, args),
        }
    }
}

#[derive(Debug)]
pub enum EvalError {
    ArgsMismatch(String),
    SomethingWentWrong, // placeholder.
    VariableIsVoid(String),
    FunctionDefinitionIsVoid(String),
    FailedToParse(ParserError),
    FailedToReadFile(String, std::io::Error),
}

pub type EvalResult = Result<Value, EvalError>;

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum Quote {
    None,
    Single,
    Eval,
}

#[derive(Debug, Clone, Eq)]
pub enum Value {
    Nil,
    T,
    Integer(Integer),
    String(String),
    Symbol(Symbol),
    Funcall(Symbol, Vec<Value>),
    List(Vec<Value>),
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Value::Nil => match other {
                Value::Nil => true,
                _ => false,
            },
            Value::T => match other {
                Value::T => true,
                _ => false,
            },
            Value::Integer(i) => match other {
                Value::Integer(j) => i == j,
                _ => false,
            },
            Value::String(s1) => match other {
                Value::String(s2) => s1 == s2,
                _ => false,
            },
            Value::Symbol(s1) => match other {
                Value::Symbol(s2) => s1 == s2,
                _ => false,
            },
            Value::Funcall(car, cdr) => match other {
                Value::Funcall(fun, args) => car == fun && cdr == args,
                _ => false,
            },
            Value::List(v1) => match other {
                Value::List(v2) => v1 == v2,
                _ => false,
            },
        }
    }
}

impl Value {
    pub fn eval(&self, environment: &mut Environment) -> EvalResult {
        match self {
            Self::Symbol(symbol) => match symbol.quote {
                Quote::Single => Ok(self.clone()),
                _ => match environment.lookup(&symbol) {
                    Some(value) => match symbol.quote {
                        Quote::None => Ok(value),
                        Quote::Eval => value.eval(environment),
                        _ => Err(EvalError::SomethingWentWrong),
                    },
                    None => Err(EvalError::VariableIsVoid(symbol.to_string())),
                },
            },
            Self::Funcall(symbol, args) => environment.call(symbol, args.to_vec()),
            Self::List(elements) => {
                let mut evaluated: Vec<Self> = Vec::new();

                for element in elements.iter() {
                    evaluated.push(element.eval(environment)?);
                }

                Ok(Self::List(evaluated))
            }
            _ => Ok(self.to_owned()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Closure {
    pub caller: String,
    scope: HashMap<String, Value>,
}

impl Closure {
    pub fn new(caller: String) -> Self {
        Self {
            caller,
            scope: HashMap::new(),
        }
    }

    pub fn put(&mut self, symbol: Symbol, value: Value) {
        self.scope.insert(symbol.name, value);
    }

    pub fn get(&self, symbol: &Symbol) -> Option<&Value> {
        self.scope.get(&symbol.name)
    }

    pub fn has(&self, symbol: &Symbol) -> bool {
        self.scope.contains_key(&symbol.name)
    }

    // Used in tests only.
    #[allow(dead_code)]
    pub fn put_str(&mut self, key: &str, value: Value) {
        let symbol = match parse(&key.to_string()).unwrap() {
            Value::Symbol(symbol) => symbol,
            _ => panic!("Not a symbol: {}", key),
        };

        self.put(symbol, value);
    }
}

pub struct Environment {
    stack: Vec<Closure>,
    functions_table: HashMap<Symbol, Function>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            stack: vec![Closure::new("top-level".into())],
            functions_table: HashMap::new(),
        }
    }

    pub fn new_configured() -> Self {
        let mut environment = Self::new();
        crate::builtins::configure(&mut environment);
        environment
    }

    pub fn top_level(&mut self) -> &mut Closure {
        self.stack.first_mut().unwrap()
    }

    pub fn current(&mut self) -> &mut Closure {
        self.stack.last_mut().unwrap()
    }

    pub fn outer(&mut self) -> &mut Closure {
        let index = self.stack.len() - 2;
        self.stack.get_mut(index).unwrap()
    }

    pub fn push_to_stack(&mut self, caller: &String) {
        self.stack.push(Closure::new(caller.to_string()))
    }

    pub fn pop(&mut self) -> Option<Closure> {
        self.stack.pop()
    }

    pub fn add_function(&mut self, key: Symbol, function: Function) {
        self.functions_table.insert(key, function);
    }

    pub fn lookup(&self, symbol: &Symbol) -> Option<Value> {
        for frame in self.stack.iter().rev() {
            if let Some(value) = frame.get(symbol) {
                return Some(value.clone());
            }
        }

        None
    }

    pub fn find_closure(&mut self, symbol: &Symbol) -> Option<&mut Closure> {
        for frame in self.stack.iter_mut().rev() {
            if frame.has(symbol) {
                return Some(frame);
            }
        }

        None
    }

    pub fn call(&mut self, symbol: &Symbol, args: Vec<Value>) -> EvalResult {
        self.push_to_stack(&symbol.name);

        let result = match self.functions_table.get(symbol).cloned() {
            Some(function) => function.call(self, args),
            None => Err(EvalError::FunctionDefinitionIsVoid(symbol.to_string())),
        };

        self.pop();

        result
    }

    pub fn eval(&mut self, buffer: &String) -> EvalResult {
        parse(buffer).map_err(EvalError::FailedToParse)?.eval(self)
    }

    pub fn eval_stdin(&mut self) -> EvalResult {
        let mut buffer = String::new();

        if let Err(err) = std::io::stdin().read_to_string(&mut buffer) {
            return Err(EvalError::FailedToReadFile("stdin".into(), err));
        }

        self.eval(&format!("(progn {})", buffer))
    }

    pub fn eval_file(&mut self, name: String) -> EvalResult {
        match std::fs::read_to_string(&name) {
            Ok(buffer) => self.eval(&format!("(progn {})", buffer)),
            Err(err) => Err(EvalError::FailedToReadFile(name, err)),
        }
    }

    // Used in `tests`.
    #[allow(dead_code)]
    pub fn eval_str(&mut self, buffer: &str) -> EvalResult {
        self.eval(&buffer.into())
    }
}

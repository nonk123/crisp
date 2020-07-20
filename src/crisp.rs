use crate::parsers::{parse, ParserError};

use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Clone, Eq, Hash)]
pub struct Symbol(String);

impl Symbol {
    pub fn from_str(string: &str) -> Self {
        Self(string.to_string())
    }

    // Used in `tests`.
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
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

#[derive(Clone)]
pub struct ArgDescriptor {
    name: Symbol,
    eval: bool,
    rest: bool,
}

impl ArgDescriptor {
    pub fn new(name: Symbol, eval: bool, rest: bool) -> Self {
        Self { name, eval, rest }
    }
}

type Builtin = fn(&mut Environment, Vec<Value>) -> EvalResult;

#[derive(Clone)]
pub struct Defun {
    body: Value,
    takes: Vec<ArgDescriptor>,
}

#[derive(Clone)]
pub enum Function {
    Builtin(Builtin),
    Defun(Defun),
}

impl Function {
    pub fn new_defun(body: Value, takes: Vec<ArgDescriptor>) -> Self {
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
        for descriptor in defun.takes.iter() {
            if descriptor.rest {
                let value = {
                    let list = Value::List(args);

                    if descriptor.eval {
                        list.eval(environment)?
                    } else {
                        list
                    }
                };

                environment.current().put(descriptor.name.clone(), value);

                break;
            } else {
                let value = {
                    let arg = match args.get(0) {
                        Some(_) => args.remove(0),
                        None => return Err(EvalError::ArgsMismatch),
                    };

                    if descriptor.eval {
                        arg.eval(environment)?
                    } else {
                        arg.clone()
                    }
                };

                environment.current().put(descriptor.name.clone(), value);
            }
        }

        let result = defun.body.eval(environment);

        result
    }

    pub fn call(&self, environment: &mut Environment, args: Vec<Value>) -> EvalResult {
        environment.push_new();

        let result = match self {
            Self::Builtin(function) => function(environment, args),
            Self::Defun(defun) => self.eval_defun(environment, defun, args),
        };

        environment.pop();

        result
    }
}

#[derive(Debug)]
pub enum EvalError {
    ArgsMismatch,
    VariableIsVoid(String),
    FunctionDefinitionIsVoid(String),
    FailedToParse(ParserError),
    FailedToReadFile(String, std::io::Error),
}

pub type EvalResult = Result<Value, EvalError>;

#[derive(Debug, Clone, Eq)]
pub enum Value {
    Nil,
    T,
    Integer(Integer),
    String(String),
    Symbol { symbol: Symbol, quoted: bool },
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
            Value::Symbol {
                symbol: s1,
                quoted: q1,
            } => match other {
                Value::Symbol {
                    symbol: s2,
                    quoted: q2,
                } => s1 == s2 && q1 == q2,
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
            Self::Symbol { symbol, quoted } => {
                if *quoted {
                    Ok(self.clone())
                } else {
                    match environment.lookup(&symbol) {
                        Some(value) => Ok(value),
                        None => Err(EvalError::VariableIsVoid(symbol.to_string())),
                    }
                }
            }
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
pub struct Closure(HashMap<Symbol, Value>);

impl Closure {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn put(&mut self, key: Symbol, value: Value) {
        self.0.insert(key, value);
    }

    // Used in tests only.
    #[allow(dead_code)]
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

    pub fn new_configured() -> Self {
        let mut environment = Self::new();
        crate::builtins::configure(&mut environment);
        environment
    }

    // Used in tests only.

    #[allow(dead_code)]
    pub fn top_level(&mut self) -> &mut Closure {
        self.stack.first_mut().unwrap()
    }

    #[allow(dead_code)]
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

    pub fn lookup(&self, symbol: &Symbol) -> Option<Value> {
        for frame in self.stack.iter().rev() {
            if let Some(value) = frame.0.get(symbol) {
                return Some(value.clone());
            }
        }

        None
    }

    pub fn find_closure(&mut self, symbol: &Symbol) -> Option<&mut Closure> {
        for frame in self.stack.iter_mut().rev() {
            if frame.0.contains_key(symbol) {
                return Some(frame);
            }
        }

        None
    }

    pub fn call(&mut self, symbol: &Symbol, args: Vec<Value>) -> EvalResult {
        self.push_new();

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

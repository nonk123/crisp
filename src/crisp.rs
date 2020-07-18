use crate::parsers::{determine_parser, ParserError};

pub type CrispInteger = i32;

#[derive(Debug)]
pub enum Value {
    Integer(CrispInteger),
    Symbol(String),
}

#[derive(Debug)]
pub enum EvalError {
    ErrorDuringParsing(ParserError),
    NoParserAvailable,
}

pub type EvalResult = Result<Value, EvalError>;

pub fn eval(buffer: String) -> EvalResult {
    if let Some(parser) = determine_parser(&buffer) {
        parser.parse(&buffer).map_err(EvalError::ErrorDuringParsing)
    } else {
        Err(EvalError::NoParserAvailable)
    }
}

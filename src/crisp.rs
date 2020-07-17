use crate::parsers::{determine_parser, ParserError};

pub type EvalTree = Vec<String>;

pub fn eval(buffer: String) -> Result<EvalTree, ParserError> {
    if let Some(parser) = determine_parser(&buffer) {
        parser.parse(&buffer)
    } else {
        Err(ParserError::NoParserAvailable)
    }
}

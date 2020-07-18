use crate::crisp::{CrispInteger, Value};

#[derive(Debug)]
pub enum ParserError {
    MalformedSymbol(String),
    MalformedInteger,
    IntegerOverflow,
}

type ParserResult = Result<Value, ParserError>;

pub trait Parser {
    fn can_parse(&self, buffer: &String) -> bool;
    fn parse(&self, buffer: &String) -> ParserResult;
}

struct IntegerParser;

impl IntegerParser {
    fn new() -> Self {
        Self
    }
}

impl Parser for IntegerParser {
    fn can_parse(&self, number: &String) -> bool {
        let mut number = number.clone();

        if vec![Some('-'), Some('+')].contains(&number.chars().nth(0)) {
            number = number[1..].to_string();
        }

        if number.is_empty() {
            return false;
        }

        for character in number.chars() {
            if !('0'..='9').contains(&character) {
                return false;
            }
        }

        true
    }

    fn parse(&self, number: &String) -> ParserResult {
        let mut number = number.clone();

        let sign: CrispInteger = {
            if number.chars().nth(0) == Some('-') {
                number = number[1..].to_string();
                -1
            } else if number.chars().nth(0) == Some('+') {
                number = number[1..].to_string();
                1
            } else {
                1
            }
        };

        let mut integer: CrispInteger = 0;

        for character in number.chars() {
            if ('0'..='9').contains(&character) {
                let value: CrispInteger = (character as u8 - b'0').into();

                // TODO: de-uglify.
                match integer.checked_mul(10) {
                    Some(result) => match result.checked_add(value) {
                        Some(result) => integer = result,
                        None => return Err(ParserError::IntegerOverflow),
                    },
                    None => return Err(ParserError::IntegerOverflow),
                }
            } else {
                return Err(ParserError::MalformedInteger);
            }
        }

        Ok(Value::Integer(integer * sign))
    }
}

pub struct SymbolParser {
    allowed_characters: Vec<char>,
}

impl SymbolParser {
    fn new() -> Self {
        let mut allowed_characters: Vec<char> = Vec::new();

        let mut add_range = |begin: u8, end: u8| {
            allowed_characters.append(&mut (begin..=end).map(char::from).collect());
        };

        add_range(b'a', b'z');
        add_range(b'A', b'Z');

        add_range(b'0', b'9');

        add_range(b'!', b'&');
        add_range(b'*', b'/');
        add_range(b':', b'@');

        allowed_characters.push('^');
        allowed_characters.push('_');
        allowed_characters.push('~');

        Self { allowed_characters }
    }
}

impl Parser for SymbolParser {
    fn can_parse(&self, symbol: &String) -> bool {
        !symbol.is_empty()
    }

    fn parse(&self, token: &String) -> ParserResult {
        let mut token = token.clone();

        if token.chars().nth(0) == Some('\'') {
            token = token[1..].to_string();
        }

        for character in token.chars() {
            if !self.allowed_characters.contains(&character) {
                return Err(ParserError::MalformedSymbol(format!(
                    "Illegal character: ({})",
                    character
                )));
            }
        }

        Ok(Value::Symbol(token.to_string()))
    }
}

pub fn determine_parser(buffer: &String) -> Option<Box<dyn Parser>> {
    let parsers: Vec<Box<dyn Parser>> = vec![
        Box::new(IntegerParser::new()),
        Box::new(SymbolParser::new()),
    ];

    for parser in parsers {
        if parser.can_parse(&buffer) {
            return Some(parser);
        }
    }

    None
}

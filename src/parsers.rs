use crate::crisp::{CrispInteger, Symbol, Value};

#[derive(Debug)]
pub enum ParserError {
    MalformedSymbol(String),
    MalformedInteger,
    IntegerOverflow,
    NoMatchingParser,
}

pub type ParserResult = Result<Value, ParserError>;

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

        if [Some('-'), Some('+')].contains(&number.chars().nth(0)) {
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

        let quoted = {
            if token.chars().nth(0) == Some('\'') {
                token = token[1..].to_string();
                true
            } else {
                false
            }
        };

        for character in token.chars() {
            if !self.allowed_characters.contains(&character) {
                return Err(ParserError::MalformedSymbol(format!(
                    "Illegal character: ({})",
                    character
                )));
            }
        }

        Ok(Value::Symbol {
            symbol: Symbol::new(&token),
            quoted,
        })
    }
}

struct ListParser;

impl ListParser {
    fn new() -> Self {
        Self
    }
}

impl Parser for ListParser {
    fn can_parse(&self, list: &String) -> bool {
        let list = {
            if list.chars().nth(0) == Some('\'') {
                list[1..].to_string()
            } else {
                list.clone()
            }
        };

        list.chars().nth(0) == Some('(') && list.chars().last() == Some(')')
    }

    fn parse(&self, list: &String) -> ParserResult {
        let mut quoted = false;

        let list = {
            if list.chars().nth(0) == Some('\'') {
                quoted = true;
                list[2..].to_string()
            } else {
                list[1..].to_string()
            }
        };

        let mut elements: Vec<Value> = Vec::new();

        let mut depth = 0;

        let mut element = String::new();

        for character in list.chars() {
            if depth == 0 && [' ', '\t', ')'].contains(&character) {
                elements.push(parse(&element)?);
                element = String::new();
            } else {
                element.push(character);
            }

            if character == '(' {
                depth += 1;
            } else if character == ')' {
                depth -= 1;
            }
        }

        Ok(Value::List { elements, quoted })
    }
}

pub fn parse(buffer: &String) -> ParserResult {
    let parsers: Vec<Box<dyn Parser>> = vec![
        Box::new(ListParser::new()),
        Box::new(IntegerParser::new()),
        Box::new(SymbolParser::new()),
    ];

    for parser in parsers {
        if parser.can_parse(&buffer) {
            return parser.parse(buffer);
        }
    }

    Err(ParserError::NoMatchingParser)
}

use crate::crisp::{Integer, Symbol, Value};

use regex::Regex;

use std::collections::HashMap;

#[derive(Debug)]
pub enum ParserError {
    MalformedInput(String),
    IntegerOverflow,
    InvalidEscapeSequence(char),
    UnmatchedParentheses,
    EmptyFuncall,
    InvalidFuncall,
    NoMatchingParser,
}

pub type ParserCheckResult = Result<(), ParserError>;
pub type ParserResult = Result<Value, ParserError>;

pub trait Parser {
    fn has_next(&self, buffer: &String) -> ParserCheckResult;
    fn parse(&self, buffer: &String) -> ParserResult;
}

struct IntegerParser {
    regex: Regex,
}

impl IntegerParser {
    fn new() -> Self {
        Self {
            regex: Regex::new(r"(?P<sign>^\+|^-|^)(?P<number>[0-9]+)$").unwrap(),
        }
    }
}

impl Parser for IntegerParser {
    fn has_next(&self, buffer: &String) -> ParserCheckResult {
        if self.regex.is_match(buffer) {
            Ok(())
        } else {
            Err(ParserError::MalformedInput(
                "Regex doesn't match".to_string(),
            ))
        }
    }

    fn parse(&self, buffer: &String) -> ParserResult {
        let captures = self.regex.captures(buffer).unwrap();

        let sign: Integer = match captures.name("sign").unwrap().as_str() {
            "-" => -1,
            _ => 1,
        };

        let mut number: Integer = 0;

        for character in captures.name("number").unwrap().as_str().chars() {
            let digit = (character as u8 - b'0').into();

            match number.checked_mul(10) {
                Some(result) => match result.checked_add(digit) {
                    Some(result) => number = result,
                    None => return Err(ParserError::IntegerOverflow),
                },
                None => return Err(ParserError::IntegerOverflow),
            }
        }

        Ok(Value::Integer(number * sign))
    }
}

pub struct SymbolParser {
    regex: Regex,
}

impl SymbolParser {
    fn new() -> Self {
        let re = r"^(?P<q>')?(?P<symbol>[a-zA-Z0-9!#-&*-/:-@^_`~|]+)$";

        Self {
            regex: Regex::new(re).unwrap(),
        }
    }
}

impl Parser for SymbolParser {
    fn has_next(&self, buffer: &String) -> ParserCheckResult {
        if self.regex.is_match(buffer) {
            Ok(())
        } else {
            Err(ParserError::MalformedInput(
                "Illegal characters in symbol name".to_string(),
            ))
        }
    }

    fn parse(&self, buffer: &String) -> ParserResult {
        let captures = self.regex.captures(buffer).unwrap();

        Ok(Value::Symbol {
            symbol: Symbol::from_str(captures.name("symbol").unwrap().as_str()),
            quoted: captures.name("q").is_some(),
        })
    }
}

struct BracketParser;

impl BracketParser {
    fn new() -> Self {
        Self
    }
}

impl Parser for BracketParser {
    fn has_next(&self, buffer: &String) -> ParserCheckResult {
        if buffer.len() < 2 {
            return Err(ParserError::MalformedInput("Too short".into()));
        }

        if !['(', '['].contains(&buffer.chars().nth(0).unwrap()) {
            return Err(ParserError::MalformedInput("Not a list".into()));
        }

        let mut matching: Vec<char> = Vec::new();

        for character in buffer.chars() {
            match character {
                '(' => matching.push(')'),
                '[' => matching.push(']'),
                ')' | ']' => {
                    if Some(character) != matching.pop() {
                        return Err(ParserError::UnmatchedParentheses);
                    }
                }
                _ => {}
            };
        }

        if matching.is_empty() {
            Ok(())
        } else {
            Err(ParserError::UnmatchedParentheses)
        }
    }

    fn parse(&self, buffer: &String) -> ParserResult {
        let buffer = buffer[1..].to_string();

        let mut elements: Vec<Value> = Vec::new();

        let mut element = String::new();

        for character in buffer.chars() {
            if [' ', '\t', '\n', '\r', ')', ']'].contains(&character) {
                // Skip random whitespaces.
                if element.is_empty() {
                    continue;
                }

                match parse(&element) {
                    Ok(value) => {
                        elements.push(value);
                        element = String::new();
                    }
                    Err(ParserError::NoMatchingParser) => element.push(character),
                    Err(err) => return Err(err),
                }
            } else {
                element.push(character);
            }
        }

        if buffer.chars().last() == Some(')') {
            if elements.is_empty() {
                return Err(ParserError::EmptyFuncall);
            }

            if let Value::Symbol { symbol, quoted } = elements.first().unwrap() {
                if !quoted {
                    let cdr = elements.iter().skip(1).cloned().collect();
                    return Ok(Value::Funcall(symbol.clone(), cdr));
                }
            }

            Err(ParserError::InvalidFuncall)
        } else {
            Ok(Value::List(elements))
        }
    }
}

struct SpecialParser {
    mappings: HashMap<&'static str, Value>,
}

impl SpecialParser {
    fn new() -> Self {
        let mut mappings: HashMap<&str, Value> = HashMap::new();

        mappings.insert("t", Value::T);
        mappings.insert("nil", Value::Nil);

        Self { mappings }
    }
}

impl Parser for SpecialParser {
    fn has_next(&self, buffer: &String) -> ParserCheckResult {
        if self.mappings.contains_key(buffer.as_str()) {
            Ok(())
        } else {
            Err(ParserError::MalformedInput(
                "Not a special token".to_string(),
            ))
        }
    }

    fn parse(&self, buffer: &String) -> ParserResult {
        Ok(self.mappings.get(buffer.as_str()).unwrap().clone())
    }
}

struct StringParser;

impl StringParser {
    fn new() -> Self {
        Self
    }
}

impl Parser for StringParser {
    fn has_next(&self, buffer: &String) -> ParserCheckResult {
        if buffer.len() >= 2
            && buffer.chars().nth(0) == Some('"')
            && buffer.chars().last() == Some('"')
        {
            Ok(())
        } else {
            Err(ParserError::MalformedInput(
                "Must be enclosed in double quotes".into(),
            ))
        }
    }

    fn parse(&self, buffer: &String) -> ParserResult {
        let mut escape_sequences: HashMap<char, char> = HashMap::new();

        escape_sequences.insert('"', '"');
        escape_sequences.insert('n', '\n');
        escape_sequences.insert('t', '\t');
        escape_sequences.insert('\\', '\\');

        let mut escaping = false;
        let mut string = String::new();

        for (index, character) in buffer[1..].chars().enumerate() {
            if escaping {
                if let Some(result) = escape_sequences.get(&character) {
                    string.push(*result);
                    escaping = false;
                } else {
                    return Err(ParserError::InvalidEscapeSequence(character));
                }
            } else if character == '\\' {
                escaping = true;
            } else if character == '"' {
                // One off because of `buffer[1..]`.
                if index == buffer.len() - 2 {
                    return Ok(Value::String(string));
                } else {
                    return Err(ParserError::MalformedInput(
                        "String literal closed early".into(),
                    ));
                }
            } else {
                string.push(character);
            }
        }

        // Never reached due to conditions in `.has_next()`.
        Err(ParserError::MalformedInput(
            "Reached the end of string literal".into(),
        ))
    }
}

pub fn parse(buffer: &String) -> ParserResult {
    let parsers: Vec<Box<dyn Parser>> = vec![
        Box::new(IntegerParser::new()),
        Box::new(SpecialParser::new()),
        Box::new(StringParser::new()),
        Box::new(SymbolParser::new()),
        Box::new(BracketParser::new()),
    ];

    for parser in parsers {
        match parser.has_next(buffer) {
            Ok(_) => return parser.parse(buffer),
            Err(_) => {}
        }
    }

    Err(ParserError::NoMatchingParser)
}

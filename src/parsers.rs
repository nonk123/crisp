use crate::crisp::EvalTree;

#[derive(Debug)]
pub enum ParserError {
    NoParserAvailable,
    MalformedToken(String),
}

type ParserResult = Result<EvalTree, ParserError>;

pub trait Parser {
    fn can_parse(&self, buffer: &String) -> bool;
    fn parse(&self, buffer: &String) -> ParserResult;
}

pub struct TokenParser {
    allowed_characters: Vec<char>,
}

impl TokenParser {
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

        TokenParser { allowed_characters }
    }
}

impl Parser for TokenParser {
    fn can_parse(&self, buffer: &String) -> bool {
        // Last resort parser.
        true
    }

    fn parse(&self, token: &String) -> ParserResult {
        for character in token.chars() {
            if !self.allowed_characters.contains(&character) {
                return Err(ParserError::MalformedToken(format!(
                    "Illegal character: ({})",
                    character
                )));
            }
        }

        Ok(vec![token.to_string()])
    }
}

pub fn determine_parser(buffer: &String) -> Option<Box<dyn Parser>> {
    for parser in vec![TokenParser::new()] {
        if parser.can_parse(&buffer) {
            return Some(Box::new(parser));
        }
    }

    None
}

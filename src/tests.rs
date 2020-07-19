use crate::crisp::{Integer, Value};

fn parse(buffer: &str) -> crate::parsers::ParserResult {
    crate::parsers::parse(&buffer.into())
}

#[test]
fn special() {
    for (buffer, value) in [("t", Value::T), ("nil", Value::Nil)].iter() {
        assert_eq!(parse(buffer).unwrap(), *value);
    }
}

#[test]
fn integer() {
    let integers: Vec<Integer> = vec![0, 1, -1, 10, -10, 99, -99];

    for i in integers.iter() {
        assert_eq!(
            parse(format!("{}", i).as_str()).unwrap(),
            Value::Integer(*i)
        );
    }

    assert_eq!(parse("+1000").unwrap(), Value::Integer(1000));

    assert!(parse("100000000000").is_err());
}

#[test]
fn symbol() {
    match parse("'hello").unwrap() {
        Value::Symbol { symbol, quoted } => {
            assert!(symbol.as_str() == "hello");
            assert!(quoted)
        }
        _ => panic!("Failed to parse 'hello as symbol"),
    }

    // Yes, that is a symbol name.
    assert!(parse("+*answer/to-the-universe=42*+").is_ok());

    // TOOD: fix failure.
    assert!(parse("'with-a space").is_err());
}

// TODO: test functions.

#[test]
fn list() {
    assert_eq!(
        parse("[t nil]").unwrap(),
        Value::List(vec![Value::T, Value::Nil])
    );

    assert_eq!(
        parse("[[t t] [nil nil]]").unwrap(),
        Value::List(vec![
            Value::List(vec![Value::T, Value::T]),
            Value::List(vec![Value::Nil, Value::Nil])
        ])
    );
}

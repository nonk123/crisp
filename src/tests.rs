use crate::crisp::{Environment, Integer, Symbol, Value};

fn parse(buffer: &str) -> crate::parsers::ParserResult {
    crate::parsers::parse(&buffer.into())
}

#[test]
fn special() {
    let tests = [("t", Value::T), ("nil", Value::Nil)];

    for (buffer, value) in tests.iter() {
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
    assert!(parse("-99999999999").is_err());
    assert!(parse("+99999999999").is_err());
}

#[test]
fn string() {
    assert_eq!(parse("\"meh\"").unwrap(), Value::String("meh".into()));
    assert_eq!(
        parse("\"Hello, World!\"").unwrap(),
        Value::String("Hello, World!".into())
    );

    assert_eq!(
        parse("[\"hello\" \"world\"]").unwrap(),
        Value::List(vec![
            Value::String("hello".into()),
            Value::String("world".into()),
        ])
    );
    assert_eq!(
        parse("[\"hello world\" \"goodbye world\"]").unwrap(),
        Value::List(vec![
            Value::String("hello world".into()),
            Value::String("goodbye world".into()),
        ])
    );

    assert_eq!(parse("\"\\\\\"").unwrap(), Value::String("\\".into()));
    assert_eq!(
        parse("\"\\\"hello\\\"\"").unwrap(),
        Value::String("\"hello\"".into())
    );
    assert_eq!(
        parse("\"hello\\nworld\"").unwrap(),
        Value::String("hello\nworld".into())
    );

    assert!(parse("\"hello").is_err());
    assert!(parse("hello\"").is_err());
    assert!(parse("\"hello\"\"").is_err());
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

    // Yes, that is a valid symbol name.
    assert!(parse("+*answer/to/the|universe=42*+").is_ok());

    assert!(parse("'with-a space").is_err());
    assert!(parse("'a 'b").is_err());

    let mut environment = Environment::new();
    let value = Value::Integer(42);

    environment.top_level().put_str("the-answer", value.clone());

    assert_eq!(environment.eval(&"the-answer".into()).unwrap(), value);
    assert_eq!(
        environment.eval(&"'the-answer".into()).unwrap(),
        Value::Symbol {
            symbol: Symbol::from_str("the-answer"),
            quoted: true,
        }
    );
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

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

    let mut environment = Environment::new_configured();
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

#[test]
fn funcall() {
    let eval = |buffer| Environment::new_configured().eval_str(buffer).unwrap();

    assert_eq!(eval("(+ 1 2 3)"), Value::Integer(6));
    assert_eq!(eval("(+ 10 -5)"), Value::Integer(5));
    assert_eq!(eval("(- 10)"), Value::Integer(-10));
    assert_eq!(eval("(* 2 -2)"), Value::Integer(-4));
    assert_eq!(eval("(/ 10 2)"), Value::Integer(5));

    assert_eq!(
        eval("(car ['a 'b 'c 10 -10 \"meh\"])"),
        Value::Symbol {
            symbol: Symbol::from_str("a"),
            quoted: true
        }
    );

    assert_eq!(
        eval("(car [[10 20] [30 40]])"),
        Value::List(vec![Value::Integer(10), Value::Integer(20)])
    );

    assert_eq!(
        eval("(cdr ['hello-world \"foo\" \"bar\"])"),
        Value::List(vec![
            Value::String("foo".into()),
            Value::String("bar".into())
        ])
    );

    assert_eq!(eval("(progn 1 2 3 4 5)"), Value::Integer(5));
    assert_eq!(eval("(progn (+ 1 2 3) (- 1 2 3))"), Value::Integer(-4));

    assert_eq!(eval("(if t 1 0)"), Value::Integer(1));
    assert_eq!(eval("(if nil 100)"), Value::Nil);
    assert_eq!(eval("(when nil 100)"), Value::Nil);
    assert_eq!(eval("(when t 100)"), Value::Integer(100));
}

#[test]
fn factorial() {
    assert_eq!(
        Environment::new_configured()
            .eval_file("std/factorial.cr".into())
            .unwrap(),
        Value::Integer(120)
    );
}

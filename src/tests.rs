use crate::crisp::{Environment, Integer, Quote, Symbol, Value};

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
    match parse("'hello") {
        Ok(Value::Symbol(symbol)) => {
            assert_eq!(symbol.as_str(), "hello");
            assert_eq!(symbol.quote, Quote::Single);
            assert_eq!(symbol.rest, false);
        }
        _ => panic!("Failed to parse 'hello as symbol"),
    }

    match parse(",bye") {
        Ok(Value::Symbol(symbol)) => {
            assert_eq!(symbol.as_str(), "bye");
            assert_eq!(symbol.quote, Quote::Eval);
            assert_eq!(symbol.rest, false);
        }
        _ => panic!("Failed to parse ,bye as symbol"),
    }

    match parse("'actually-no...") {
        Ok(Value::Symbol(symbol)) => {
            assert_eq!(symbol.as_str(), "actually-no");
            assert_eq!(symbol.quote, Quote::Single);
            assert_eq!(symbol.rest, true);
        }
        _ => panic!("Failed to parse 'actually-no... as symbol"),
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
        Value::Symbol(Symbol::new("the-answer".into(), Quote::Single, false))
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
        Value::Symbol(Symbol::new("a".into(), Quote::Single, false))
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

    assert_eq!(eval("(if nil 100)"), Value::Nil);
    assert_eq!(eval("(if nil 1 0)"), Value::Integer(0));
    assert_eq!(eval("(if t 1 0)"), Value::Integer(1));
}

#[test]
fn factorial() {
    let mut environment = Environment::new_configured();

    environment.top_level().put_str("input", Value::Integer(5));

    assert_eq!(
        environment.eval_file("test/factorial.cr".into()).unwrap(),
        Value::Integer(120)
    );
}

#[test]
fn fibonacci() {
    let mut environment = Environment::new_configured();

    environment.eval_file("test/fibonacci.cr".into()).unwrap();

    assert_eq!(
        environment.eval_str("(fibonacci 5)").unwrap(),
        Value::Integer(5)
    );
}

#[test]
fn quoted_args() {
    let mut environment = Environment::new_configured();

    assert_eq!(
        environment.eval_file("test/quoted-args.cr".into()).unwrap(),
        Value::Integer(120)
    );
}

#[test]
fn rest_args() {
    let mut environment = Environment::new_configured();

    environment.eval_file("test/rest-args.cr".into()).unwrap();

    assert_eq!(
        environment.eval_str("(rcar 1 2 3)").unwrap(),
        Value::Integer(1)
    );
    assert_eq!(
        environment.eval_str("(rcdr 1 2 3)").unwrap(),
        Value::List(vec![Value::Integer(2), Value::Integer(3),])
    );

    assert!(environment.eval_str("(defun buggy [a... b...])").is_err());
}

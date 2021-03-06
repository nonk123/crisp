use crate::crisp::{Environment, EvalError, EvalResult, Function, Integer, Symbol, Value};

pub fn configure(environment: &mut Environment) {
    let functions: Vec<(&str, fn(&mut Environment, Vec<Value>) -> EvalResult)> = vec![
        ("progn", progn),
        ("debug", debug),
        ("if", if_),
        ("while", while_),
        ("set", set),
        ("let", let_),
        ("=", eq),
        ("/=", neq),
        ("+", add),
        ("-", sub),
        ("*", mul),
        ("/", div),
        ("car", car),
        ("cdr", cdr),
        ("defun", defun),
    ];

    for (name, function) in functions {
        environment.add_function(Symbol::from_str(name), Function::new_builtin(function));
    }
}

fn mismatch<R>(environment: &mut Environment, reason: &str) -> Result<R, EvalError> {
    Err(EvalError::ArgsMismatch(format!(
        "`{}': {}",
        environment.current().caller,
        reason
    )))
}

fn list_arg(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() == 1 {
        let value = args[0].eval(environment)?;

        match value {
            Value::List(_) => Ok(value),
            _ => mismatch(environment, "This function takes a list".into()),
        }
    } else {
        mismatch(
            environment,
            "This function takes exactly one list argument".into(),
        )
    }
}

fn reduce<T, C: Fn(Value) -> Option<T>, R: Fn(T, T) -> T>(
    environment: &mut Environment,
    starting: Value,
    args: Vec<Value>,
    conversion: C,
    operation: R,
) -> Result<T, EvalError> {
    let mut starting = match conversion(starting.eval(environment)?) {
        Some(some) => some,
        None => return mismatch(environment, "Couldn't convert the starting value".into()),
    };

    for value in args.iter() {
        match conversion(value.eval(environment)?) {
            Some(converted) => starting = operation(starting, converted),
            None => {
                return mismatch(
                    environment,
                    format!("Couldn't convert argument {:?}", value).as_str(),
                )
            }
        }
    }

    Ok(starting)
}

fn reduce_car_cdr<T, C: Fn(Value) -> Option<T>, R: Fn(T, T) -> T>(
    environment: &mut Environment,
    args: Vec<Value>,
    conversion: C,
    operation: R,
) -> Result<T, EvalError> {
    let car = match args.first() {
        Some(value) => value,
        None => return mismatch(environment, "No car in the list".into()),
    };

    let cdr = args[1..].to_vec();

    reduce(environment, car.clone(), cdr, conversion, operation)
}

fn make_progn(args: Vec<Value>) -> Value {
    Value::Funcall(Symbol::from_str("progn"), args)
}

fn some_args(environment: &mut Environment, args: Vec<Value>) -> Result<Vec<Value>, EvalError> {
    if args.len() == 0 {
        mismatch(environment, "This function takes one or more args".into())
    } else {
        Ok(args)
    }
}

fn is_nil(value: &Value) -> bool {
    match value {
        Value::Nil => true,
        Value::List(elements) => elements.is_empty(),
        Value::String(string) => string.is_empty(),
        _ => false,
    }
}

fn progn(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.is_empty() {
        return Ok(Value::Nil);
    }

    for arg in args[..args.len() - 1].iter() {
        arg.eval(environment)?;
    }

    args.last().unwrap_or(&Value::Nil).eval(environment)
}

fn debug(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.is_empty() {
        return Ok(Value::Nil);
    }

    let mut last = args.first().unwrap().clone();

    for arg in args {
        last = arg.eval(environment)?;
        println!("{:?}", last);
    }

    Ok(last.clone())
}

fn if_(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let condition = match args.get(0) {
        Some(value) => !is_nil(&value.eval(environment)?),
        None => return mismatch(environment, "This function takes a condition".into()),
    };

    let if_true = match args.get(1) {
        Some(value) => value,
        None => return mismatch(environment, "This function takes a 'when' parameter".into()),
    };

    if condition {
        if_true.eval(environment)
    } else {
        progn(environment, args[2..].to_vec())
    }
}

fn while_(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() < 2 {
        return mismatch(
            environment,
            "This function takes a condition and loop body".into(),
        );
    }

    loop {
        let condition = args.first().unwrap();
        let action = make_progn(args[1..].to_vec());

        if is_nil(&condition.eval(environment)?) {
            return Ok(Value::Nil);
        }

        action.eval(environment)?;
    }
}

fn symbol_binding(
    environment: &mut Environment,
    symbol: Value,
    value: Value,
) -> Result<(Symbol, Value), EvalError> {
    match symbol.eval(environment)? {
        Value::Symbol(symbol) => Ok((symbol, value.eval(environment)?)),
        _ => mismatch(environment, "First argument must be a symbol".into()),
    }
}

fn symbol_binding_argslist(
    environment: &mut Environment,
    args: Vec<Value>,
) -> Result<(Symbol, Value), EvalError> {
    if args.len() != 2 {
        return mismatch(
            environment,
            "This function takes a symbol and its value".into(),
        );
    }

    symbol_binding(
        environment,
        args.first().unwrap().clone(),
        args.last().unwrap().clone(),
    )
}

fn set(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let (symbol, value) = symbol_binding_argslist(environment, args)?;

    if let Some(closure) = environment.find_closure(&symbol) {
        closure.put(symbol, value.clone());
    } else {
        environment.top_level().put(symbol, value.clone());
    }

    Ok(value)
}

fn let_(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let (symbol, value) = symbol_binding_argslist(environment, args)?;
    environment.outer().put(symbol, value.clone());
    Ok(value)
}

fn eq(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    reduce_car_cdr(
        environment,
        args,
        |x| Some(x),
        |x, y| match x == y {
            true => Value::T,
            false => Value::Nil,
        },
    )
}

fn neq(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    match eq(environment, args)? {
        Value::T => Ok(Value::Nil),
        _ => Ok(Value::T),
    }
}

fn to_integer(value: Value) -> Option<Integer> {
    match value {
        Value::Integer(i) => Some(i),
        _ => None,
    }
}

fn add(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let args = some_args(environment, args)?;

    Ok(Value::Integer(reduce(
        environment,
        Value::Integer(0),
        args,
        to_integer,
        |x, y| x + y,
    )?))
}

fn sub(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() == 1 {
        match args.first().unwrap() {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            _ => mismatch(
                environment,
                "This function takes one or more integer values".into(),
            ),
        }
    } else {
        Ok(Value::Integer(reduce_car_cdr(
            environment,
            args,
            to_integer,
            |x, y| x - y,
        )?))
    }
}

fn mul(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let args = some_args(environment, args)?;

    Ok(Value::Integer(reduce(
        environment,
        Value::Integer(1),
        args,
        to_integer,
        |x, y| x * y,
    )?))
}

fn div(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    Ok(Value::Integer(reduce_car_cdr(
        environment,
        args,
        to_integer,
        |x, y| x / y,
    )?))
}

fn car(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    match list_arg(environment, args)? {
        Value::List(elements) => elements.first().unwrap_or(&Value::Nil).eval(environment),
        _ => mismatch(environment, "This function takes a list".into()),
    }
}

fn cdr(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    match list_arg(environment, args)? {
        Value::List(elements) => {
            Value::List(elements.iter().cloned().skip(1).collect()).eval(environment)
        }
        _ => mismatch(environment, "This function takes a list".into()),
    }
}

fn defun(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() < 2 {
        return mismatch(
            environment,
            "This function takes a function name, arg descriptor, and optional body".into(),
        );
    }

    let name = match args.first().unwrap() {
        Value::Symbol(symbol) => symbol,
        _ => return mismatch(environment, "The first argument must be a symbol".into()),
    };

    let body = make_progn(args[2..].to_vec());

    let mut takes: Vec<Symbol> = Vec::new();

    let args_list = match args.get(1).unwrap() {
        Value::List(args) => args,
        _ => {
            return mismatch(
                environment,
                "The second argument must be a list of symbols".into(),
            )
        }
    };

    for arg in args_list.iter() {
        match arg {
            Value::Symbol(symbol) => {
                match takes.last() {
                    Some(next) => {
                        if symbol.rest && next.rest {
                            return mismatch(environment, "Only one rest arg is allowed");
                        }
                    }
                    _ => {}
                }

                takes.push(symbol.clone());
            }
            _ => return mismatch(environment, "Args list contains a non-symbol value"),
        }
    }

    environment.add_function(name.clone(), Function::new_defun(body, takes));

    Ok(Value::Nil)
}

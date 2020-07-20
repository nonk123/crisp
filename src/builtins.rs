use crate::crisp::{Environment, EvalError, EvalResult, Integer, Symbol, Value};

pub fn configure(environment: &mut Environment) {
    environment.add_function_str("progn", progn);
    environment.add_function_str("debug", debug);

    environment.add_function_str("if", if_);
    environment.add_function_str("when", when);

    environment.add_function_str("while", while_);

    environment.add_function_str("let", let_);
    environment.add_function_str("set", set);

    environment.add_function_str("=", eq);
    environment.add_function_str("/=", neq);

    environment.add_function_str("+", add);
    environment.add_function_str("-", sub);
    environment.add_function_str("*", mul);
    environment.add_function_str("/", div);

    environment.add_function_str("car", car);
    environment.add_function_str("cdr", cdr);
}

fn list_arg(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() == 1 {
        let value = args[0].eval(environment)?;

        match value {
            Value::List(_) => Ok(value),
            _ => Err(EvalError::ArgsMismatch),
        }
    } else {
        Err(EvalError::ArgsMismatch)
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
        None => return Err(EvalError::ArgsMismatch),
    };

    for value in args.iter() {
        match conversion(value.eval(environment)?) {
            Some(converted) => starting = operation(starting, converted),
            None => return Err(EvalError::ArgsMismatch),
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
        None => return Err(EvalError::ArgsMismatch),
    };

    let cdr = args[1..].to_vec();

    reduce(environment, car.clone(), cdr, conversion, operation)
}

fn make_progn(args: Vec<Value>) -> Value {
    Value::Funcall(Symbol::from_str("progn"), args)
}

fn some_args(args: Vec<Value>) -> Result<Vec<Value>, EvalError> {
    if args.len() == 0 {
        Err(EvalError::ArgsMismatch)
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
    for arg in args {
        println!("{:?}", arg.eval(environment)?);
    }

    Ok(Value::Nil)
}

fn if_(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let condition = match args.get(0) {
        Some(value) => !is_nil(&value.eval(environment)?),
        None => return Err(EvalError::ArgsMismatch),
    };

    let if_true = match args.get(1) {
        Some(value) => value,
        None => return Err(EvalError::ArgsMismatch),
    };

    if condition {
        if_true.eval(environment)
    } else {
        progn(environment, args[2..].to_vec())
    }
}

fn when(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() < 2 {
        return Err(EvalError::ArgsMismatch);
    }

    let condition = args.first().unwrap().clone();
    let action = make_progn(args[1..].to_vec());

    if_(environment, vec![condition, action])
}

fn while_(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() < 2 {
        return Err(EvalError::ArgsMismatch);
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
        Value::Symbol { symbol, quoted: _ } => {
            let value = value.eval(environment)?;
            Ok((symbol, value))
        }
        _ => Err(EvalError::ArgsMismatch),
    }
}

fn symbol_binding_argslist(
    environment: &mut Environment,
    args: Vec<Value>,
) -> Result<(Symbol, Value), EvalError> {
    if args.len() != 2 {
        return Err(EvalError::ArgsMismatch);
    }

    symbol_binding(
        environment,
        args.first().unwrap().clone(),
        args.last().unwrap().clone(),
    )
}

fn let_(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let (symbol, value) = symbol_binding_argslist(environment, args)?;
    environment.outer().put(symbol, value.clone());
    Ok(value)
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
    Ok(Value::Integer(reduce(
        environment,
        Value::Integer(0),
        some_args(args)?,
        to_integer,
        |x, y| x + y,
    )?))
}

fn sub(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() == 1 {
        match args.first().unwrap() {
            Value::Integer(i) => Ok(Value::Integer(-i)),
            _ => Err(EvalError::ArgsMismatch),
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
    Ok(Value::Integer(reduce(
        environment,
        Value::Integer(1),
        some_args(args)?,
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
        _ => Err(EvalError::ArgsMismatch),
    }
}

fn cdr(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    match list_arg(environment, args)? {
        Value::List(elements) => {
            Value::List(elements.iter().cloned().skip(1).collect()).eval(environment)
        }
        _ => Err(EvalError::ArgsMismatch),
    }
}

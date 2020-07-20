use crate::crisp::{
    ArgDescriptor, Environment, EvalError, EvalResult, Function, Integer, Symbol, Value,
};

pub fn configure(environment: &mut Environment) {
    let functions: Vec<(&str, fn(&mut Environment, Vec<Value>) -> EvalResult)> = vec![
        ("progn", progn),
        ("debug", debug),
        ("if", if_),
        ("when", when),
        ("while", while_),
        ("set", set),
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

fn defun(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    if args.len() < 2 {
        return Err(EvalError::ArgsMismatch);
    }

    let name = match args.first().unwrap() {
        Value::Symbol { symbol, quoted: _ } => symbol,
        _ => return Err(EvalError::ArgsMismatch),
    };

    let body = make_progn(args[2..].to_vec());

    let mut takes: Vec<ArgDescriptor> = Vec::new();

    let args_list = match args.get(1).unwrap() {
        Value::List(args) => args,
        _ => return Err(EvalError::ArgsMismatch),
    };

    for arg in args_list.iter() {
        match arg {
            Value::Symbol { symbol, quoted } => {
                takes.push(ArgDescriptor::new(symbol.clone(), !quoted, false))
            }
            _ => return Err(EvalError::ArgsMismatch),
        }
    }

    environment.add_function(name.clone(), Function::new_defun(body, takes));

    Ok(Value::Nil)
}

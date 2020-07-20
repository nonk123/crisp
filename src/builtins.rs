use crate::crisp::{Environment, EvalError, EvalResult, Integer, Value};

pub fn configure(environment: &mut Environment) {
    environment.add_function_str("+", add);
    environment.add_function_str("-", sub);
    environment.add_function_str("*", mul);
    environment.add_function_str("/", div);

    environment.add_function_str("car", car);
    environment.add_function_str("cdr", cdr);
}

fn list_arg(args: Vec<Value>) -> EvalResult {
    if args.len() == 1 {
        match args[0] {
            Value::List(_) => Ok(args[0].clone()),
            _ => Err(EvalError::ArgsMismatch),
        }
    } else {
        Err(EvalError::ArgsMismatch)
    }
}

fn reduce<F: Fn(Integer, Integer) -> Integer>(
    environment: &mut Environment,
    mut starting: Integer,
    args: Vec<Value>,
    operation: F,
) -> EvalResult {
    for value in args.iter() {
        match value.eval(environment)? {
            Value::Integer(i) => starting = operation(starting, i),
            _ => return Err(EvalError::ArgsMismatch),
        }
    }

    Ok(Value::Integer(starting))
}

fn some_args(args: Vec<Value>) -> Result<Vec<Value>, EvalError> {
    if args.len() == 0 {
        Err(EvalError::ArgsMismatch)
    } else {
        Ok(args)
    }
}

fn add(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    reduce(environment, 0, some_args(args)?, |x, y| x + y)
}

fn sub(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    reduce(environment, 0, some_args(args)?, |x, y| x - y)
}

fn mul(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    reduce(environment, 1, some_args(args)?, |x, y| x * y)
}

fn div(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    let car = match args.first() {
        Some(value) => match value {
            Value::Integer(i) => i,
            _ => return Err(EvalError::ArgsMismatch),
        },
        None => return Err(EvalError::ArgsMismatch),
    };

    let cdr = args[1..].to_vec();

    reduce(environment, *car, cdr, |x, y| x / y)
}

fn car(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    match list_arg(args)?.eval(environment)? {
        Value::List(elements) => elements.first().unwrap_or(&Value::Nil).eval(environment),
        _ => Err(EvalError::ArgsMismatch),
    }
}

fn cdr(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
    match list_arg(args)?.eval(environment)? {
        Value::List(elements) => {
            Value::List(elements.iter().cloned().skip(1).collect()).eval(environment)
        }
        _ => Err(EvalError::ArgsMismatch),
    }
}

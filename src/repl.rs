use std::io;
use std::io::Write;

use crate::crisp::{eval, Closure, Environment, EvalError, EvalResult, Value};

fn read_line() -> io::Result<String> {
    let mut buffer = String::new();

    io::stdin().read_line(&mut buffer)?;

    if buffer.ends_with("\n") {
        buffer.pop();

        if buffer.ends_with("\r") {
            buffer.pop();
        }
    }

    Ok(buffer)
}

pub fn mainloop() -> io::Result<()> {
    let mut environment = Environment::new();

    let top = environment.top_level();
    top.put_str("answer", Value::Integer(42));
    top.put_str("hmm?", Value::Integer(15663));

    let mut bot = Closure::new();
    bot.put_str("answer", Value::Integer(43));
    bot.put_str("real-answer", Value::Integer(42));
    environment.push_to_stack(bot);

    fn car(environment: &mut Environment, args: Vec<Value>) -> EvalResult {
        if args.len() != 1 {
            Err(EvalError::ArgsMismatch)
        } else {
            let arg = args.first().unwrap().eval(environment)?;

            match arg {
                Value::List {
                    elements,
                    quoted: _,
                } => elements.first().unwrap_or(&Value::Nil).eval(environment),
                _ => Err(EvalError::ArgsMismatch),
            }
        }
    }

    environment.add_function_str("car", car);

    loop {
        print!("> ");
        io::stdout().flush()?;

        let input = read_line()?;

        if ["exit", "quit"].contains(&input.as_str()) {
            println!("Goodbye!");
            return Ok(());
        }

        match eval(&mut environment, input) {
            Ok(value) => println!("{:?}", value),
            Err(error) => println!("{:?}", error),
        }
    }
}

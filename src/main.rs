mod builtins;
mod crisp;
mod parsers;
mod repl;

#[cfg(test)]
mod tests;

use crate::crisp::{Environment, EvalError};

#[derive(Debug)]
enum RuntimeError {
    IO(std::io::Error),
    Eval(EvalError),
}

fn main() -> Result<(), RuntimeError> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        return repl::mainloop().map_err(RuntimeError::IO);
    }

    let mut environment = Environment::new_configured();

    for file in args {
        let result = if file.as_str() == "-" {
            environment.eval_stdin()
        } else {
            environment.eval_file(file)
        };

        result.map_err(RuntimeError::Eval)?;
    }

    Ok(())
}

use std::io;
use std::io::Write;

use crate::crisp::Environment;

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
    let mut environment = Environment::new_configured();

    loop {
        print!("> ");
        io::stdout().flush()?;

        let input = read_line()?;

        if ["exit", "quit"].contains(&input.as_str()) {
            println!("Goodbye!");
            return Ok(());
        }

        match environment.eval(&input) {
            Ok(value) => println!("{:?}", value),
            Err(error) => println!("{:?}", error),
        }
    }
}

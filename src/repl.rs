use std::io;
use std::io::Write;

use crate::crisp;

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
    loop {
        print!("> ");
        io::stdout().flush()?;

        let input = read_line()?;

        if vec!["exit", "quit"].contains(&input.as_str()) {
            println!("Goodbye!");
            return Ok(());
        }

        match crisp::eval(input) {
            Ok(tree) => println!("{}", tree.join(" ")),
            Err(error) => println!("{:?}", error),
        }
    }
}

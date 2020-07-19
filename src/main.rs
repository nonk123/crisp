mod crisp;
mod parsers;
mod repl;

#[cfg(test)]
mod tests;

fn main() -> std::io::Result<()> {
    repl::mainloop()
}

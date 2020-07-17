mod crisp;
mod parsers;
mod repl;

fn main() -> std::io::Result<()> {
    repl::mainloop()
}

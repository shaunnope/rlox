use std::env;
use std::process;

#[cfg(test)]
mod tests;

fn main() {
  let _ = rtlox::parse_args(env::args()).unwrap_or_else(|err| {
    eprintln!("Problem parsing arguments: {err}");
    process::exit(1);
  });
}

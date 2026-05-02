use std::env::args;
use std::io::{self, Write, stdin};

use lox::SyntaxError;
use lox::error_handling::LoxError;
use lox::scanner::Scanner;

fn main() {
  let args: Vec<String> = args().collect();
  if args.len() > 2 {
    println!("Usage: lox [script]");
    std::process::exit(64)
  }
  if args.len() == 1 {
    repl();
    return;
  }
  run_file(&args[1]).expect("Cannot read such file");
}

fn repl() {
  let stdin = stdin();

  loop {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut line = String::new();
    let bytes_read = stdin.read_line(&mut line).expect("Failed to read");
    if bytes_read == 0 {
      break;
    }
    let line = line.trim_end();
    if line.is_empty() {
      break;
    }
    match run(&line.to_string()) {
      Ok(_) => {}
      Err(err) => {
        err.report();
      }
    }
  }
}

fn run_file(path: &str) -> io::Result<()> {
  let buf = std::fs::read_to_string(path)?;
  match run(&buf) {
    Ok(_) => {}
    Err(err) => {
      err.report();
      std::process::exit(65);
    }
  }
  Ok(())
}

fn run(source: &str) -> Result<(), LoxError> {
  let scanner = Scanner::new(source);
  scanner.scan_tokens()?;
  Ok(())
}

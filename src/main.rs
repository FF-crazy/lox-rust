use std::env::args;
use std::io::{self, Write, stdin};

use lox::error_handling::LoxError;
use lox::interpreter::Interpreter;
use lox::parser::Parser;
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
  let mut interpreter = Interpreter::new();

  loop {
    print!("> ");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut line = String::new();
    let bytes_read = stdin.read_line(&mut line).expect("Failed to read");
    if bytes_read == 0 {
      break;
    }
    // The interpreter retains source-backed tokens between REPL submissions.
    let line: &'static str = Box::leak(line.trim_end().to_owned().into_boxed_str());
    if line.is_empty() {
      continue;
    }
    match run(line, &mut interpreter) {
      Ok(_) => {}
      Err(err) => {
        if let LoxError::Syntax(origin_err) = err {
          let line: &'static str = Box::leak(try_format_input(line).into_boxed_str());
          match run(&line, &mut interpreter) {
            Ok(()) => {}
            Err(LoxError::Syntax(_)) => origin_err.report(),
            Err(other) => other.report(),
          }
        } else {
          err.report();
        }
      }
    }
  }
}

fn try_format_input(input: &str) -> String {
  let mut res = input.to_string();
  if !input.ends_with(";") {
    res = format!("{};", res);
  }
  if !input.starts_with("print ") {
    res = format!("print {}", res);
  }
  res
}

fn run_file(path: &str) -> io::Result<()> {
  let buf = std::fs::read_to_string(path)?;
  let mut interpreter = Interpreter::new();
  match run(&buf, &mut interpreter) {
    Ok(_) => {}
    Err(err) => {
      err.report();
      std::process::exit(65);
    }
  }
  Ok(())
}

fn run<'src>(source: &'src str, interpreter: &mut Interpreter<'src>) -> Result<(), LoxError<'src>> {
  let scanner = Scanner::new(source);
  let tokens = scanner.scan_tokens()?;
  let parser = Parser::new(tokens);
  let stmts = parser.parse()?;
  interpreter.interpret(&stmts)?;
  Ok(())
}

use std::env::args;
use std::fs::File;
use std::io::{self, Read, Write, stdin};

fn main() {
  let args: Vec<String> = args().collect();
  if args.len() != 2 && args.len() != 1 {
    println!("Usage: lox [script]");
    return;
  }
  if args.len() == 1 {
    repl();
    return;
  }
  let read_result = File::open(&args[1]);
  if let Ok(mut lox_file) = read_result {
    let mut buf = String::new();
    lox_file.read_to_string(&mut buf).unwrap();
    print!("{}", buf);
  } else {
    println!("No such File");
  }
}

fn repl() {
  loop {
    print!(">  ");
    io::stdout().flush().expect("Failed to flush stdout");
    let mut buf = String::new();
    stdin().read_line(&mut buf).expect("Failed to read");
    println!("{}", buf);
  }
}

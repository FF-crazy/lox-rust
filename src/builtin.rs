use crate::object::{NativeFunction, Object};

pub struct Builtin;

impl Builtin {
  fn lox_abs(mut args: Vec<Object>) -> Result<Object, String> {
    match args.pop() {
      Some(Object::Number(n)) => Ok(Object::Number(n.abs())),
      Some(other) => Err(format!(
        "abs expects a Number, got '{}'",
        other.get_type_name()
      )),
      None => Err("abs expects 1 argument".to_string()),
    }
  }

  fn lox_print(mut args: Vec<Object>) -> Result<Object, String> {
    match args.pop() {
      Some(obj) => {
        print!("{}", obj);
        Ok(Object::Nil)
      }
      None => Err("abs expects 1 argument".to_string()),
    }
  }

  fn lox_println(args: Vec<Object>) -> Result<Object, String> {
    let res = Self::lox_print(args);
    println!();
    res
  }

  pub fn output_builtin<'src>() -> Vec<(&'static str, Object<'src>)> {
    let mut res = Vec::new();
    res.push((
      "abs",
      Object::Native(NativeFunction::new("abs", 1, Self::lox_abs)),
    ));
    res.push((
      "print",
      Object::Native(NativeFunction::new("print", 1, Self::lox_print)),
    ));
    res.push((
      "println",
      Object::Native(NativeFunction::new("println", 1, Self::lox_println)),
    ));
    res
  }
}

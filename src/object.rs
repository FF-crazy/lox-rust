use derive_more::Display;

#[derive(Display, Debug, Clone, PartialEq, PartialOrd)]
pub enum Object {
  Number(f64),
  String(String),
  Nil,
  Boolean(bool),
}

impl Object {
  pub fn get_type_name(&self) -> &'static str {
    match self {
      Object::Number(_) => "Number",
      Object::String(_) => "String",
      Object::Boolean(_) => "Boolean",
      Object::Nil => "Nil",
    }
  }
}

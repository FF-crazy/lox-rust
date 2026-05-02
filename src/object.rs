use derive_more::Display;

#[derive(Display, Debug, Clone)]
pub enum Object {
  Number(f64),
  String(String),
  Nil,
  Boolean(bool),
}

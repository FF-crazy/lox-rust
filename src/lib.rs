pub mod error_handling;
pub mod expr;
pub mod object;
pub mod scanner;
pub mod token;
pub mod parser;
pub mod interpreter;

pub use error_handling::SyntaxError;

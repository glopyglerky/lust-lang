//! Kernel mirror of the ts-rust `gostd` module (only the files the JSON
//! kernel uses: `errors` and `strconv`, copied verbatim).

pub mod errors;
pub mod strconv;

pub use errors::GoError;

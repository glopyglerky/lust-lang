mod implementation { pub fn answer() -> i64 { 42 } }
#[deprecated(note = "use a different API")]
pub use implementation::answer;

// An FnOnce closure stored in a field is called by value.
struct H<F: FnOnce() -> String> { f: F }
fn main() { let s = String::from("abc"); let h = H { f: move || s }; let out = (h.f)(); println!("{}", out); }

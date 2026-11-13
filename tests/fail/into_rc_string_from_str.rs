// expect: L0104
// rustc E0277: `Rc<String>: From<&str>` does not hold (only `Rc<str>` has it).
fn main() { let r: std::rc::Rc<String> = "x".into(); println!("{}", r); }

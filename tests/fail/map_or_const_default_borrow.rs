// expect: L0733
// rustc "lifetime may not live long enough": the closure returns a
// borrow of the argument, which is not 'static (the constant default
// does not hide it).
pub struct Rule { pub own: String }
fn name_of<'a>(r: Option<&'a Rule>) -> &'static str { r.map_or("", |r| r.own.as_str()) }
fn main() { let r = Rule { own: String::from("a") }; println!("{}", name_of(Some(&r))); }

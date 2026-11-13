// expect: L0001
// rustc E0658: the arguments of println! and the other format macros take
// no outer attribute (vec! elements do).
fn main() { println!("{}", #[cfg(all())] 1); }

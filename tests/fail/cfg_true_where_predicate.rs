// expect: L0001
// rustc E0658: a where predicate takes no outer attribute.
fn w<T>(t: T) -> T where #[cfg(all())] T: Clone { t }
fn main() { println!("{}", w(1)); }

fn g<A: std::fmt::Debug>(a: A) -> String { format!("{:?}", a) }
#[derive(Debug)] struct W<A, B>(A, B);
fn main() { println!("{:?} {:?}", g::<>(1u8), W::<>(1u8, 2u16)); }

struct S<T>(T);
impl<T> S<T> { fn w<U>(&self, _u: U) -> usize { std::mem::size_of::<U>() } }
fn main() { println!("{}", S(1u8).w::<u64>(3)); }

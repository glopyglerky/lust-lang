struct S<T>(T);
impl<T> S<T> { fn neg<U: std::ops::Not<Output = U>>(&self, u: U) -> U { !u } }
fn main() { println!("{}", S(0u8).neg::<u8>(0)); }

trait K { const V: u32; }
impl K for u8 { const V: u32 = 1; }
impl<T: K> K for Vec<T> { const V: u32 = T::V + 10; }
fn g<T: K>() -> u32 { T::V * 2 }
fn main() { println!("{} {}", g::<u8>(), <Vec<u8> as K>::V); }

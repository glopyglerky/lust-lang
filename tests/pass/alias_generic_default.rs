// An alias type param default (`type P<T = u32>`).
type P<T = u32> = (T, T);
fn main() { let p: P = (1, 2); let q: P<char> = ('a', 'b'); println!("{} {} {:?}", p.0, p.1, q); }

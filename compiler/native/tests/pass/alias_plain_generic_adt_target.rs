// A non-generic alias to a generic struct (`type Q = P<u32>`).
struct P<T> { t: T }
type Q = P<u32>;
fn main() { let q: Q = P { t: 3 }; println!("{}", q.t); }

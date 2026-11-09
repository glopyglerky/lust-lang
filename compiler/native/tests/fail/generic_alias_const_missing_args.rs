// expect: L0110
// rustc E0107: missing generics for the alias.
type Twice<T> = (T, T);
const X: Twice = (1, 2);
fn main() { println!("{}", X.0); }

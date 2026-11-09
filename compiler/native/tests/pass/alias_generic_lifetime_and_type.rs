// An alias with a lifetime and a type param.
type Pair<'a, T> = (&'a T, &'a T);
fn first<'a, T>(p: Pair<'a, T>) -> &'a T { p.0 }
fn main() { let a = 1; let b = 2; println!("{}", first((&a, &b))); }

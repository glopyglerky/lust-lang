// feature: lifetimes.static_anon
// form: a named std carrier with '_ in a return type, slice Iter<'_>
fn items(v: &[i32]) -> std::slice::Iter<'_, i32> { v.iter() }
fn main() { let v = vec![1, 2]; println!("{}", items(&v).count()); }

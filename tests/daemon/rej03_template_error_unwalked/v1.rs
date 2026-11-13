// hunt6 01:11: a collect that fails on a template error only (show) after
// a bound edit: main was a read-set walk candidate (it names twice) and
// kept its passing mark for a walk that never ran. v3 edits only show's
// body; warm must still reject twice(vec![1]) under T: Copy (E0277).
// v4 puts the Clone bounds back.
#![allow(unused)]
fn show<T: Clone>(t: T) -> T { println!("{:?}", std::any::type_name::<T>()); t }
fn twice<T: Clone>(t: T) -> (T, T) { (t.clone(), t) }
fn main() { println!("{:?}", twice(vec![1])); println!("end"); }

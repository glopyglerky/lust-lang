// A ManuallyDrop const in a pattern (core makes it structural), guards
// through Deref, and its type_name.
use std::mem::ManuallyDrop;
const C: ManuallyDrop<i32> = ManuallyDrop::new(3);
fn main() {
    let m = ManuallyDrop::new(3);
    match m { C => println!("c"), _ => println!("other") }
    let x = ManuallyDrop::new(Some(2));
    match x { ref v if v.is_some() => println!("{:?}", v), _ => {} }
    let opt = Some(ManuallyDrop::new(5));
    match opt { Some(ref m) if **m > 4 => println!("big"), _ => println!("small") }
    println!("{}", std::any::type_name::<ManuallyDrop<u8>>());
}

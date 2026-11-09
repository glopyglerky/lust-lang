// ManuallyDrop's Ord and Hash forward to the value: sort, max, map keys.
use std::collections::HashMap;
use std::mem::ManuallyDrop;
fn main() {
    let mut v = vec![ManuallyDrop::new(3), ManuallyDrop::new(1), ManuallyDrop::new(2)];
    v.sort();
    println!("{:?} {:?}", v, v.iter().max());
    let mut m = HashMap::new();
    m.insert(ManuallyDrop::new(7u32), "seven");
    println!("{:?} {}", m.get(&ManuallyDrop::new(7u32)), m.contains_key(&ManuallyDrop::new(8)));
    let a = ManuallyDrop::new(1.5f64);
    println!("{:?} {}", a.partial_cmp(&ManuallyDrop::new(f64::NAN)), a < ManuallyDrop::new(2.0));
}

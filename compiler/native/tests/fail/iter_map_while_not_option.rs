// expect: L0110
// rustc E0308: the map_while closure must return an Option.
fn main() { let v = vec![1u32]; let c: Vec<u32> = v.iter().map_while(|x| *x).collect(); println!("{:?}", c); }

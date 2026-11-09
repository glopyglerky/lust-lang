fn pair<A, B>(a: A, b: B) -> (A, B) { (a, b) }
fn main() { let x = pair::<u8, u16>(255, 1); println!("{:?} {:?}", x, pair(1u8, 2u8)); }

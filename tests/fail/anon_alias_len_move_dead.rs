// expect: L0301
// rustc E0382: an array length in a type alias is borrow-checked as a body.
struct NC;
type A = [u8; {
    if false {
        let s = NC;
        let t = s;
        let u = s;
    }
    3
}];
fn main() {
    println!("{}", std::mem::size_of::<A>());
}

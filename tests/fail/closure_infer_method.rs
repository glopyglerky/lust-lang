// expect: L0610
fn main() {
    let f = |x| x.len();
    println!("{}", f(vec![1u8]));
}

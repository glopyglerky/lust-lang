// `nth` reaches i8::MAX, then the second forward step overflows.
fn main() {
    let mut r = 5i8..;
    println!("start");
    let x = r.nth(122);
    println!("{:?}", x);
}

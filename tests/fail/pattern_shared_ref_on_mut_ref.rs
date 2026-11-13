// expect: L0110
fn main() {
    let &x = &mut 5u8;
    println!("{}", x);
}

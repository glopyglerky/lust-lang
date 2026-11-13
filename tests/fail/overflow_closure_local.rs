// expect: L0540
fn main() {
    let f = || {
        let a: u8 = 255;
        a + 1
    };
    println!("{}", f());
}

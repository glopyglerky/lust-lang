// expect: L0540
fn main() {
    let c = || {
        let x: u8 = 255;
        x + 1
    };
    println!("{}", c());
}

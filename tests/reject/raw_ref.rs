// expect: L0820
fn main() {
    let x = 3;
    let p = &raw const x;
    println!("{}", !p.is_null());
}

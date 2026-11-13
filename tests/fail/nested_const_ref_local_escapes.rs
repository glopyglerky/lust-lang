// expect: L0307
// rustc E0597: a local of the initializer block dies at the block's end.
fn main() {
    const C: &u32 = {
        let a = 5u32;
        &a
    };
    println!("{}", C);
}

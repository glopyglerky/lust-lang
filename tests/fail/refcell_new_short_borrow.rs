// expect: L0306
// rustc E0597: the cell holds `&x`, and `x` ends with the block.
fn main() {
    let c;
    {
        let x = 5u8;
        c = std::cell::RefCell::new(&x);
    }
    println!("{:?}", c);
}

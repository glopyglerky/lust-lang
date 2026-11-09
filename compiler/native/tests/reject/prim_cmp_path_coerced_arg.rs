// expect: L0641
// Valid Rust (prints `Less`): the argument `&&i32` deref-coerces to
// `&i32`. lust lowers a builtin comparison by trait path as the method
// form, which reads each argument at its written type, so a coerced
// argument is a reject (it crashed before).
fn main() {
    let y = 1;
    let x: &&i32 = &&y;
    println!("{:?}", <i32 as Ord>::cmp(x, &2));
}

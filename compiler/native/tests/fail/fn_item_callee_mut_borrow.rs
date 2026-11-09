// expect: L0308
// rustc E0596: the callee `&mut c` borrows an immutable binding, though
// the fn item call does not read it.
fn seven() -> i32 { 7 }
fn main() {
    let c = seven;
    println!("{}", (&mut c)());
}

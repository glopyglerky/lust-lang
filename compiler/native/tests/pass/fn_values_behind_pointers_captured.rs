// Closures that call fn values behind a Box borrow them, as rustc's
// callee autoderef does; a move closure takes them.
fn seven() -> i32 { 7 }
fn main() {
    let p: fn() -> i32 = seven;
    let w = Box::new(&p);
    let mut c = seven;
    let m = &mut c;
    let f = || w() + (*m)();
    println!("{}", f());
    let g = move || w();
    println!("{}", g());
}

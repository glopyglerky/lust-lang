// expect: L0101
fn main() {
    let x: Option<i32> = None;
    let Some(v) = x else { println!("{}", v); return; };
    println!("{}", v);
}

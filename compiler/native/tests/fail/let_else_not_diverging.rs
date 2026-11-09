// expect: L0110
fn main() {
    let x: Option<i32> = Some(1);
    let Some(v) = x else { println!("hi") };
    println!("{}", v);
}

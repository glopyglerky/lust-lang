// expect: L0001
fn main() {
    let c = true;
    let o: Option<i32> = Some(2);
    let Some(x) = if c { o } else { None } else { return };
    println!("{}", x);
}

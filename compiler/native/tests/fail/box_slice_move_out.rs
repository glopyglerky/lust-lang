// expect: L0305
fn main() {
    let d: Box<[String]> = vec![String::from("a")].into_boxed_slice();
    let s = d[0];
    println!("{}", s);
}

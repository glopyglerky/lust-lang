// expect: L0306
fn main() {
    let f;
    {
        let s = String::from("short");
        f = || s.len();
    }
    println!("{}", f());
}

// expect: L0306
fn main() {
    let r;
    {
        let x = String::from("short");
        r = &x;
    }
    println!("{}", r);
}

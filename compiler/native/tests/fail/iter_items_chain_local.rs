// expect: L0306
// rustc E0597: a `chain` item comes from either side.
fn main() {
    let r;
    {
        let a = vec![String::from("a")];
        let b = vec![String::from("b")];
        r = a.iter().chain(b.iter()).last();
    }
    println!("{:?}", r);
}

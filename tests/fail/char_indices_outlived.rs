// expect: L0306
// rustc E0597: char_indices borrows the String.
fn main() {
    let c;
    {
        let s = String::from("ab");
        c = s.char_indices();
    }
    println!("{}", c.count());
}

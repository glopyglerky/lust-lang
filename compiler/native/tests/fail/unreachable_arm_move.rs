// expect: L0301
// rustc E0382: the unreachable arm's move still counts for the use after the match.
fn main() {
    let v = String::from("a");
    match v {
        _ => println!("a"),
        s => drop(s),
    }
    println!("{}", v);
}

// expect: L0820
fn main() {
    let s = String::from("seen");
    let c = || {
        let _ = s;
        1
    };
    println!("{} {}", c(), s);
}

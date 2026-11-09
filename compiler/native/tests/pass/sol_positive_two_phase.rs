// Sol review probe `positive_two_phase`: rustc accepts.
fn main() {
    let mut v = Vec::new();
    v.push(v.len());
    println!("{}", v.len());
}

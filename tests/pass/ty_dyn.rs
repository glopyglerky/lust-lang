// `dyn Display` (a std fmt trait as the principal).
fn show(x: &dyn std::fmt::Display) {
    println!("{}", x);
}

fn main() {
    show(&3);
}

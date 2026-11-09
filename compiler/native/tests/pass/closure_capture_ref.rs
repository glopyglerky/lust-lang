// A non-move closure captures by shared reference: the owner stays usable.
fn main() {
    let name = String::from("lust");
    let n = 3u32;
    let greet = || format!("{} x{}", name, n);
    println!("{}", greet());
    println!("{}", greet());
    println!("{} {}", name, n);
}

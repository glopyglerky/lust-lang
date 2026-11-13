fn main() {
    let n = 0u32;
    let d: &dyn base::T = &n;
    println!("{} {}", d.open(), d.secret());
}

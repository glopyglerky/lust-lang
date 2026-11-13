fn main() {
    println!("{}", cfg!(feature = "a"));
    #[cfg(feature = "b")]
    let _x: u32 = "no";
}

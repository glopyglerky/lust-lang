// expect: L0505
// rustc E0425: `format` lives in alloc (std::fmt), not in core::fmt.
fn main() {
    let s = core::fmt::format(format_args!("x"));
    println!("{}", s);
}

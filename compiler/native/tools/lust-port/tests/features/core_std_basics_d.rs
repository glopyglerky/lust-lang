// feature: core.std_basics
// form: primitive From and TryFrom paths, as in i64::from and u32::try_from in ts-rust
fn main() {
    let a = i64::from(3i32);
    let b = u32::try_from(5usize).unwrap_or(0);
    let c = usize::from(2u16);
    println!("{} {} {}", a, b, c);
}

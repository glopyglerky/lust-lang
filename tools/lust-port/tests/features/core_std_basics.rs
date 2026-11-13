// feature: core.std_basics
// form: mem::take and swap, cmp helpers and str methods, the common std surface
fn main() {
    let mut s = String::from("hi");
    let taken = std::mem::take(&mut s);
    let (mut a, mut b) = (1, 2);
    std::mem::swap(&mut a, &mut b);
    println!("{} {} {} {}", taken.len(), a + b, std::cmp::max(3, 7), "x".repeat(2).len());
}

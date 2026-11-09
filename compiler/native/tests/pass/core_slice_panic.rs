// panic: slice index starts at 3 but ends at 2
fn main() {
    let a = [1, 2, 3];
    let s = &a[3..2];
    println!("{}", s.len());
}

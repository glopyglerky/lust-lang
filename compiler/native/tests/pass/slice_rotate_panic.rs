// panic: assertion failed: k <= self.len()
// rotate_right past the length panics with core's assert at core's site (R4).
fn main() {
    let n = std::env::args().count();
    let mut v = vec![1, 2, 3];
    v.rotate_right(n + 1);
    println!("{v:?}");
    v.rotate_right(n + 3);
    println!("{v:?}");
}

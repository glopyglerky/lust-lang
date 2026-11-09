// A for loop's item drops at the end of its own iteration, before the
// next `next` call: whole for `_`, the unbound rest for a partial pattern,
// after the pattern's bindings. The body here makes no call.
struct D(i32);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop D({})", self.0);
    }
}
struct It(i32);
impl Iterator for It {
    type Item = (D, D);
    fn next(&mut self) -> Option<(D, D)> {
        if self.0 == 0 {
            return None;
        }
        self.0 -= 1;
        println!("next {}", self.0);
        Some((D(self.0), D(self.0 + 10)))
    }
}
fn main() {
    let mut k = 0;
    for _ in It(2) {
        k += 1;
    }
    println!("{k}");
    for (a, _) in It(2) {
        k += a.0;
    }
    println!("{k}");
    for (_, ref b) in It(2) {
        k += b.0;
    }
    println!("{k}");
    for _ in (0..2).map(|i| D(i + 7)) {
        k += 1;
    }
    println!("{k}");
}

// expect: L0612
// rustc E0277: rev needs DoubleEndedIterator, which C does not implement.
struct C(u32);

impl Iterator for C {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0)
        }
    }
}

fn main() {
    let v: Vec<u32> = C(3).rev().collect();
    println!("{v:?}");
}

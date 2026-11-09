// expect: L0612
// rustc E0277: a sum of u32 items is not a u8.
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
    let s: u8 = C(3).sum();
    println!("{s}");
}

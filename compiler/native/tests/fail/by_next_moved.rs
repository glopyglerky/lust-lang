// expect: L0301
// rustc E0382: the adapter moves the user iterator.
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
    let c = C(3);
    let v: Vec<u32> = c.map(|x| x).collect();
    println!("{:?} {v:?}", c.0);
}

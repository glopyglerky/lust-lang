// expect: L0734
// rustc: `impl` item signature doesn't match `trait` item signature. The
// trait promises a 'static result; the impl returns a borrow of self.
trait Read {
    fn get(&self) -> &'static i32;
}

struct S(i32);

impl Read for S {
    fn get(&self) -> &i32 {
        &self.0
    }
}

fn main() {
    let s = S(1);
    println!("{}", s.get());
}

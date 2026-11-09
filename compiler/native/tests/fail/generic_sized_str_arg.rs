// expect: L0612
// rustc E0277: `T: L` holds for `str`, but `T` is also implicitly
// `Sized`, which the call site must prove.
trait L {
    fn l(&self) -> usize;
}

impl L for str {
    fn l(&self) -> usize {
        self.len()
    }
}

fn g<T: L>(t: &T) -> usize {
    t.l()
}

fn main() {
    println!("{}", g("four"));
}

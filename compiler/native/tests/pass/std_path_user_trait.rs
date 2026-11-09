// A user trait implemented for the unsized std::path::Path (W-SHIM):
// `&self` is the fat path value; a ?Sized generic formats it.
use std::fmt::Debug;
use std::path::Path;

trait Depth {
    fn depth(&self) -> usize;
}

impl Depth for Path {
    fn depth(&self) -> usize {
        let mut n = 0;
        for _ in self.components() {
            n += 1;
        }
        n
    }
}

fn show<T: ?Sized + Debug>(t: &T) {
    println!("{:?}", t);
}

fn main() {
    for s in ["", "/", "a/b/c", "./x/../y/"] {
        let p = Path::new(s);
        println!("{} {}", p.depth(), p.to_path_buf().depth());
        show(p);
    }
    let pair = [Path::new("a"), Path::new("b")];
    println!("{:?} {}", pair, pair[0] < pair[1]);
}

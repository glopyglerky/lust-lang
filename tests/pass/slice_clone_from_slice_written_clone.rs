// clone_from_slice on a Copy type with a written Clone: core runs that impl
// per element (its clone_from when written, else clone), after the length
// check, also through a generic fn.
#[derive(Copy, Debug)]
struct C(u8);
impl Clone for C {
    fn clone(&self) -> C {
        println!("clone {}", self.0);
        C(self.0 + 1)
    }
}
#[derive(Copy, Debug)]
struct F(u8);
impl Clone for F {
    fn clone(&self) -> F {
        println!("F clone {}", self.0);
        *self
    }
    fn clone_from(&mut self, src: &F) {
        println!("F clone_from {} <- {}", self.0, src.0);
        self.0 = src.0 * 2;
    }
}
#[derive(Copy)]
struct G<T: Copy + std::fmt::Debug>(T);
impl<T: Copy + std::fmt::Debug> Clone for G<T> {
    fn clone(&self) -> G<T> {
        println!("G clone {:?}", self.0);
        *self
    }
}
fn copy_in<T: Clone>(a: &mut [T], b: &[T]) {
    a.clone_from_slice(b);
}
fn main() {
    let mut a = [C(0), C(0)];
    a.clone_from_slice(&[C(5), C(7)]);
    println!("{:?}", a);
    let mut v = vec![C(0), C(0), C(0)];
    copy_in(&mut v, &[C(1), C(2), C(3)]);
    println!("{:?}", v);
    let mut f = [F(1), F(2)];
    copy_in(&mut f, &[F(10), F(20)]);
    println!("{:?}", f);
    let mut g = [G('a'), G('b')];
    g[..1].clone_from_slice(&[G('z')]);
    println!("{} {}", g[0].0, g[1].0);
    let mut e: [C; 0] = [];
    e.clone_from_slice(&[]);
    println!("{}", e.len());
}

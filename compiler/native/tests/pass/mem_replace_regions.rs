// `mem::replace` and `mem::swap` trade values through the `&mut`, whose
// own borrow reaches neither value.
trait T {
    fn f(&self) -> u8;
}

struct A;

impl T for A {
    fn f(&self) -> u8 {
        1
    }
}

struct S {
    once: Option<Box<dyn T>>,
}

impl S {
    fn take(&mut self) -> Option<Box<dyn T>> {
        std::mem::replace(&mut self.once, None)
    }
}

fn main() {
    let k = 1u8;
    let mut v: &u8 = &k;
    let l = 2u8;
    let old = std::mem::replace(&mut v, &l);
    println!("{} {}", v, old);
    let mut a = 3u8;
    let mut b = 4u8;
    std::mem::swap(&mut a, &mut b);
    println!("{} {}", a, b);
    let mut s = S { once: Some(Box::new(A)) };
    let got = s.take();
    println!("{} {}", got.is_some(), s.once.is_none());
}

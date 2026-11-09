use std::fmt;

#[derive(Debug, Clone, PartialEq)]
struct Outer {
    a: Inner,
    n: i32,
}
struct Inner(i32);
impl fmt::Debug for Inner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>", self.0)
    }
}

impl PartialEq for Inner {
    fn eq(&self, o: &Self) -> bool {
        self.0 % 3 == o.0 % 3
    }
}
impl Clone for Inner {
    fn clone(&self) -> Self {
        Inner(self.0 + 200)
    }
}

fn twice<T: Clone + fmt::Debug>(x: &T) -> String {
    format!("{:?}", x.clone())
}
fn report(o: &Outer) -> String {
    let c = o.clone();
    format!("{:?} {:?} {}", o, c, *o == c)
}
fn main() {
    let o = Outer { a: Inner(1), n: 2 };
    println!("{}", report(&o));
    println!("{}", twice(&o));
}

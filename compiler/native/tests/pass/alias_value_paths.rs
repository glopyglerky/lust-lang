// Variant calls, struct literals, struct patterns, and associated fns
// through type aliases, including an alias of a generic enum (its args
// are the alias's).
#[derive(Debug)]
enum E<T> {
    A(T),
    B { x: T },
    C,
}
#[derive(Debug)]
struct S<T> {
    a: T,
}
struct U;
type EA = E<u8>;
type SA = S<i64>;
type UA = U;
impl<T> E<T> {
    fn mk(t: T) -> Self {
        E::A(t)
    }
}
impl U {
    fn v(&self) -> u8 {
        3
    }
}
fn main() {
    let e = EA::A(3);
    let f = EA::B { x: 9 };
    let g = EA::C;
    let s = SA { a: -4 };
    println!("{:?} {:?} {:?} {:?}", e, f, g, s);
    match e {
        EA::A(n) => println!("a {}", n),
        EA::B { x } => println!("b {}", x),
        EA::C => println!("c"),
    }
    if let EA::B { x } = f {
        println!("{}", x);
    }
    let SA { a } = s;
    println!("{}", a);
    println!("{:?}", EA::mk(5));
    let u = UA {};
    let UA {} = u;
    println!("{}", U.v());
}

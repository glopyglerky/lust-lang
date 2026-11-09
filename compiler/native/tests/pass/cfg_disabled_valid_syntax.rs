#[cfg(any())]
fn f(x: NoSuchType) -> AlsoMissing {
    let y: i32 = "not a number";
    missing_fn(x, y)
}

struct S {
    #[cfg(any())]
    a: Missing,
    b: i32,
}

enum E {
    #[cfg(any())]
    A(Missing),
    B,
}

struct T;
impl T {
    #[cfg(any())]
    fn g(&self) -> Missing {
        missing_fn()
    }
}

#[cfg(any())]
use missing_crate::Thing;

#[cfg(any())]
mod m {
    pub fn h(x: Missing) -> Missing {
        x
    }
}

fn main() {
    #[cfg(any())]
    let z: Missing = missing_fn();
    let s = S { b: 41 };
    let v = match E::B {
        #[cfg(any())]
        E::A(_) => 0,
        E::B => 1,
    };
    println!("{}", s.b + v);
}

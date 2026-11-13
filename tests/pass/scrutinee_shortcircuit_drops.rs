// A `match` scrutinee is not a terminating scope: its temporaries live to
// the end of the enclosing statement. Each operand of `&&` and `||` is
// one: its temporaries drop right after it.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn len(d: &D) -> usize {
    d.0.len()
}
fn wrap(n: usize) -> D {
    println!("wrap {}", n);
    D("w")
}
fn main() {
    let x = (match mk("a").0.len() { n => n }, mk("b"));
    println!("tuple {}", x.0);
    let n = len(&wrap(match mk("a").0.len() { n => n }));
    println!("arg {}", n);
    let y = { let _y = mk("y"); match mk("a").0.len() { n => n } };
    println!("tail {}", y);
    let z = match match mk("a").0.len() { n => mk("b").0.len() + n } { m => m };
    println!("nested {}", z);
    let p = mk("a").0.len() == 1 && mk("b").0.len() == 1;
    let q = mk("a").0.len() == 2 || mk("b").0.len() == 1;
    let r = mk("a").0.len() == 1 && mk("b").0.len() == 1 && mk("c").0.len() == 1;
    println!("logic {} {} {}", p, q, r);
    if mk("a").0.len() == 1 && mk("b").0.len() == 1 {
        println!("in");
    }
    let mut i = 0;
    while mk("c").0.len() + i < 3 {
        i += 1;
    }
    match mk("a").0.len() {
        n => println!("stmt {}", n),
    };
    println!("end {}", x.1.0);
}

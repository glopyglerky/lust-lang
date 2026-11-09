// Drop order of `let` initializer temporaries, as rustc schedules them: a
// temporary drops at the end of its statement, before the temporaries of
// its own expression; a destructured value's leftover parts drop at the
// end of the `let`.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct P {
    a: D,
    b: D,
}
struct Q {
    pair: (D, D),
}
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn mkp() -> P {
    println!("mkp");
    P { a: D("pa"), b: D("pb") }
}
fn mkq() -> Q {
    println!("mkq");
    Q { pair: (D("q0"), D("q1")) }
}
fn wrap(d: &D) -> D {
    println!("wrap {}", d.0);
    D("w")
}
fn len(d: &D) -> usize {
    d.0.len()
}
impl D {
    fn s(&self) -> &str {
        self.0
    }
}
fn main() {
    {
        let (a, _) = (mk("v"), wrap(&mk("x")));
        println!("after {}", a.0);
    }
    {
        let (a, _) = mkq().pair;
        println!("place in value {}", a.0);
    }
    {
        let x = (mk("a"), mk("b")).0;
        let y = mkp().a;
        println!("moved field {} {}", x.0, y.0);
    }
    {
        let _ = mk("t").0;
        let _ = (mk("a"), mk("b")).0;
        let _ = mkp().a;
        println!("wild places");
    }
    {
        let _ = (mk("v"), wrap(&mk("x")));
        println!("wild value");
    }
    {
        let n = wrap(&mk("x")).0.len();
        let m = wrap(&mk("y")).s().len();
        let k = len(&wrap(&mk("z")));
        println!("receivers {} {} {}", n, m, k);
    }
    {
        println!("fmt {}", wrap(&mk("x")).0);
        let b = wrap(&mk("x")).0 == mk("y").0;
        println!("eq {}", b);
    }
    {
        let x = { let _y = mk("y"); mk("t") };
        let z = loop { let _y = mk("y2"); break mk("t2"); };
        println!("values {} {}", x.0, z.0);
    }
    println!("end");
}

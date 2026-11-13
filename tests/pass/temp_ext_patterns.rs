// A `let` pattern with an explicit `ref` binding extends its whole
// initializer to the end of the block (rustc `is_binding_pat`); other
// destructuring patterns drop the initializer's leftover parts at the end
// of the statement. `let _ = &temp()` still extends the borrowed temporary.
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
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn mkp() -> P {
    println!("mkp");
    P { a: D("pa"), b: D("pb") }
}
fn main() {
    {
        let ref x = mk("t");
        let ref mut y = mk("u");
        y.0 = "u2";
        println!("ref {} {}", x.0, y.0);
    }
    {
        let (ref x, y) = (mk("t1"), mk("t2"));
        println!("tuple ref {} {}", x.0, y.0);
    }
    {
        let (ref a, _) = (mk("v"), mk("w"));
        println!("tuple ref wild {}", a.0);
    }
    {
        let (a, _) = (mk("v"), mk("w"));
        println!("tuple wild {}", a.0);
    }
    {
        let (a, _) = (&mk("v"), mk("w"));
        println!("tuple borrow wild {}", a.0);
    }
    {
        let P { ref a, .. } = mkp();
        println!("struct ref {}", a.0);
    }
    {
        let P { a, .. } = mkp();
        println!("struct val {}", a.0);
    }
    {
        let ref x @ D(_) = mk("t");
        println!("ref at {}", x.0);
    }
    {
        let ref x = mkp().a;
        println!("ref field {}", x.0);
    }
    {
        let _ = &mk("t");
        println!("wild ref");
    }
    {
        let _ = mk("t");
        println!("wild value");
    }
    {
        let (_, _b) = (&mk("t"), mk("u"));
        println!("wild tuple");
    }
    println!("end");
}

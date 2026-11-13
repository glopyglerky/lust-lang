// A destructuring `let` (or `let...else`) over a temporary drops the
// parts the pattern does not move at the end of the statement (before
// the `else` block runs); a `ref` binding extends the temporary to the
// end of the block.
struct D(&'static str);
impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}
struct H(D, D);
enum One {
    V(D, D),
}
fn mk() -> H {
    H(D("m1"), D("m2"))
}
fn mkd(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn lets() {
    let H(x, _) = H(D("x"), D("y"));
    println!("got {}", x.0);
    let (a, _) = (D("a"), D("b"));
    println!("got {}", a.0);
    let (ref c, _) = (D("c"), D("d"));
    println!("got {}", c.0);
    let H(e, _) = mk();
    println!("got {}", e.0);
    let H(_, _) = mk();
    println!("after wild");
    let One::V(f, _) = One::V(D("f"), D("g"));
    println!("got {}", f.0);
    println!("end lets");
}
fn let_else(on: bool) {
    let Some(x) = (mkd("t"), if on { Some(3) } else { None }).1 else {
        println!("else");
        return;
    };
    println!("x {}", x);
    let Some(d) = (mkd("u"), Some(mkd("v"))).1 else { return };
    println!("d {}", d.0);
    let (Some(ref a), _) = (Some(mkd("a")), mkd("b")) else { return };
    println!("a {}", a.0);
    println!("end let_else");
}
fn main() {
    lets();
    let_else(true);
    let_else(false);
}

// `S::from(x)` and `u32::from(x)` where x's type is known only after
// the call (the vec's element type comes from a later push): body end
// selects core's identity impl or a numeric widening, a move or a cast.
struct O {
    v: u8,
}
struct S {
    n: u8,
}
impl Drop for S {
    fn drop(&mut self) {
        println!("drop {}", self.n);
    }
}
impl From<O> for S {
    fn from(o: O) -> S {
        S { n: o.v }
    }
}
fn main() {
    let mut u = Vec::new();
    let mut w = Vec::new();
    let mut first = true;
    loop {
        if !first {
            let s = S::from(u.pop().unwrap());
            println!("{}", s.n);
            S::from(u.pop().unwrap());
            let x = u32::from(w.pop().unwrap());
            println!("{}", x + 1);
            break;
        }
        first = false;
        u.push(S { n: 7 });
        u.push(S { n: 8 });
        w.push(250u8);
    }
    let t = S::from(O { v: 9 });
    println!("{}", t.n);
}

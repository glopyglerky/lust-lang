// Temporary lifetime extension (Rust reference, "Temporary lifetime
// extension"): the operand of an extending borrow in a `let` initializer
// lives to the end of the block, through struct, tuple, array, and cast
// operands, constructor arguments, block tails, and `if`/`match` arms.
// It drops after the binding and before earlier locals.
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
struct S<'a> {
    r: &'a D,
}
struct W<'a>(&'a D);
enum E<'a> {
    V(&'a D),
    N,
}
mod m {
    pub struct H<'a>(pub &'a super::D);
}
trait Tr {
    fn name(&self) -> &str;
}
impl Tr for D {
    fn name(&self) -> &str {
        self.0
    }
}
fn mk(n: &'static str) -> D {
    println!("mk {}", n);
    D(n)
}
fn mkp() -> P {
    println!("mkp");
    P { a: D("pa"), b: D("pb") }
}
fn mkv() -> Vec<D> {
    println!("mkv");
    vec![D("v0"), D("v1")]
}
fn mkb(n: &'static str) -> Box<D> {
    println!("mkb {}", n);
    Box::new(D(n))
}
fn main() {
    {
        let _a = D("a");
        let x = &mk("t");
        println!("ref call {}", x.0);
        let _b = D("b");
    }
    {
        let _a = D("a");
        let p = &mkp().a;
        println!("field of temp {}", p.0);
    }
    {
        let s = S { r: &mk("t") };
        println!("struct lit {}", s.r.0);
    }
    {
        let t = (&mk("t1"), &mk("t2"));
        println!("tuple {} {}", t.0.0, t.1.0);
    }
    {
        let t = [&mk("t1"), &mk("t2")];
        println!("array {} {}", t[0].0, t[1].0);
    }
    {
        let w = W(&mk("t"));
        let o = Some(&mk("u"));
        let h = m::H(&mk("v"));
        let e = E::V(&mk("w"));
        let n = E::N;
        println!("ctors {} {} {}", w.0.0, o.unwrap().0, (h.0).0);
        if let (E::V(d), E::N) = (e, n) {
            println!("variant {}", d.0);
        }
    }
    {
        let x = { let _y = D("y"); &mk("t") };
        println!("block tail {}", x.0);
    }
    {
        let c = mkv().len() == 2;
        let x = if c { &mk("t1") } else { &mk("t2") };
        let y = match c { true => { let _q = D("q"); &mk("u1") } false => &mk("u2") };
        println!("if match {} {}", x.0, y.0);
    }
    {
        let x = &mk("t") as &dyn Tr;
        let y: &dyn Tr = &mk("u");
        let z: (&dyn Tr,) = (&mk("v"),);
        println!("dyn {} {} {}", x.name(), y.name(), z.0.name());
    }
    {
        let x = &(mk("t0"), mk("t1")).1;
        let y = &mkv()[1];
        let z = &*mkb("t2");
        let w = &mkb("t3").0;
        println!("projections {} {} {} {}", x.0, y.0, z.0, w);
    }
    {
        let x = (mk("v"), &mk("t"));
        println!("owned and borrowed {} {}", x.0.0, x.1.0);
    }
    {
        let x = &&mk("t");
        let y = &*&mk("u");
        let z = &mut mk("w");
        z.0 = "z";
        println!("nested {} {} {}", x.0, y.0, z.0);
    }
    {
        let t: &str = &String::from("hi");
        let s = &String::from("hello")[1..];
        let v: &[i32] = &vec![1, 2, 3];
        let a: &[D] = &[mk("a0"), mk("a1")];
        println!("coercions {} {} {} {}", t, s, v.len(), a.len());
    }
    println!("end");
}

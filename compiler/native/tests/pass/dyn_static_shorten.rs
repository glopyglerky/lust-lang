// A `'static` object lifetime shortens in covariant positions and at a
// top-level `&mut dyn` coercion; in a body, rustc infers `'static`.
trait T { fn read(&self) -> u32; }
struct S { n: u32 }
impl T for S { fn read(&self) -> u32 { self.n } }
impl dyn T { fn more(&self) -> u32 { self.read() + 1 } fn more_mut(&mut self) -> u32 { self.read() + 2 } }
static S0: S = S { n: 4 };
fn take(d: &dyn T) -> u32 { d.read() }
fn take_mut(d: &mut dyn T) -> u32 { d.read() }
fn out(x: &S) -> &dyn T { let d: &dyn T = x; let _ = d.more(); d }
fn out_mut(x: &mut S) -> &mut dyn T { let d: &mut dyn T = x; let _ = d.more_mut(); d }
fn out_vec(x: &S) -> Vec<&dyn T> { let d: &dyn T = x; let _ = d.more(); let v = vec![d]; v }
fn main() {
    let d: &(dyn T + 'static) = &S0;
    let mut s = S { n: 3 };
    println!("{} {}", take(d), take_mut(&mut s));
    let m: &mut (dyn T + 'static) = &mut s;
    println!("{}", take_mut(m));
    let o: Option<&(dyn T + 'static)> = Some(d);
    let p: Option<&dyn T> = o;
    let t: (&(dyn T + 'static), u8) = (d, 1);
    let u: (&dyn T, u8) = t;
    let a: [&(dyn T + 'static); 1] = [d];
    let b: [&dyn T; 1] = a;
    let e: &&(dyn T + 'static) = &d;
    let f: &&dyn T = e;
    println!("{} {} {} {}", p.unwrap().read(), u.0.read(), b[0].read(), f.read());
    let mut k = S { n: 5 };
    let g: &mut dyn T = &mut k;
    let h: &mut (dyn T + 'static) = g;
    println!("{}", h.more_mut());
    outs();
}

// A separate body: lust's `'static` proof is per body, and these calls
// return dyn values with the caller's object lifetime.
fn outs() {
    let mut q = S { n: 6 };
    let z = S { n: 7 };
    println!("{} {} {}", out(&z).read(), out_mut(&mut q).read(), out_vec(&z)[0].read());
}

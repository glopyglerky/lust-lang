// The bounds of an impl Trait return change: callers that used a bound
// that is gone must fail warm as they do cold (hunt2 daemon_rpit_bound_removed).
trait Sh { fn area(&self) -> u32; }
#[derive(Clone)]
struct Sq(u32);
impl Sh for Sq { fn area(&self) -> u32 { self.0 } }
fn mk() -> impl Sh + Clone { Sq(3) }
fn main() {
    let c = mk().clone();
    println!("{}", c.area());
}

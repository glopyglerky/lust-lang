// A call through a const or static that holds a fn pointer (static S: fn(u32) -> u32 = f;
// S(1)), next to the forms that already worked (an array item, a field, a local copy).
fn f(x: u32) -> u32 { x + 1 }
fn g(x: u32) -> u32 { x * 2 }
static S: fn(u32) -> u32 = f;
const C: fn(u32) -> u32 = g;
static TABLE: [fn(u32) -> u32; 2] = [f, g];
struct Ops { op: fn(u32) -> u32 }
static OPS: Ops = Ops { op: g };
fn main() {
    println!("{} {}", S(1), C(5));
    println!("{} {}", TABLE[1](3), (OPS.op)(4));
    let local = S;
    println!("{}", local(10));
}

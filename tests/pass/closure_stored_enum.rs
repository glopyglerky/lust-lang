// A closure in an enum variant.
enum Op<F> { Apply(F), Skip }
fn run<F: Fn(i32) -> i32>(op: Op<F>, x: i32) -> i32 { match op { Op::Apply(f) => f(x), Op::Skip => x } }
fn main() { println!("{} {}", run(Op::Apply(|x| x * 10), 2), run(Op::<fn(i32) -> i32>::Skip, 3)); }

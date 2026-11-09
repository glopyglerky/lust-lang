// Fn pointer types with inferred parts unify part by part, and a fn
// item coerces to a fn pointer whose types are still being inferred.
struct ViaFn<T>(fn(T) -> i32);

fn twice(x: u8) -> i32 {
    x as i32 * 2
}
fn app<T>(f: fn(T) -> i32, x: T) -> i32 {
    f(x)
}
fn pick<A, B>(f: fn(A) -> B, a: A) -> B {
    f(a)
}
fn neg(x: i64) -> i64 {
    -x
}

fn main() {
    let a: ViaFn<u8> = ViaFn(twice);
    let f: fn(u8) -> i32 = twice;
    let b = ViaFn(f);
    let fs = vec![f, twice];
    println!("{} {} {}", (a.0)(4), (b.0)(5), app(twice, 6));
    println!("{} {}", pick(neg, 3), fs.len());
}

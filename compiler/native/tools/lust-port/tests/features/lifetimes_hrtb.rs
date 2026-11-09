// feature: lifetimes.hrtb
// form: explicit for<'a> on a fn pointer type and on an Fn bound, as in ts-rust ast/synthetic
fn apply(f: for<'a> fn(&'a i32) -> i32, x: &i32) -> i32 { f(x) }
fn apply2<F: for<'a> Fn(&'a i32) -> i32>(f: F, x: &i32) -> i32 { f(x) }
fn main() { let v = 5; println!("{} {}", apply(|n| *n + 1, &v), apply2(|n| *n * 2, &v)); }

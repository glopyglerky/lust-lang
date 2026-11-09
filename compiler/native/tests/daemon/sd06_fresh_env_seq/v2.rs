use std::fmt::Debug;
fn pick<T: Copy + Debug>(x: T) -> T { x }
fn helper(x: i64) -> i64 where u32: Copy { x as i64 * 2 }
fn other(y: i64) -> i64  { y + 1 }
fn user() -> i64 { helper(3) + pick(1) }
fn main() { println!("{} {} {:?}", user(), other(helper(4)), pick(2)); }

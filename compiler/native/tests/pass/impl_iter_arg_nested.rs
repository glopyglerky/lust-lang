// Chains over `impl Iterator` arguments passed on to other generic fns,
// in `for` and `while let` loops, and after a `next` call.
fn inner(it: impl Iterator<Item = u64>) -> u64 { it.sum() }
fn outer(it: impl Iterator<Item = u32>) -> u64 { inner(it.map(|x| x as u64 * 2)) }
fn loop_it(it: impl Iterator<Item = i8>) { for (i, x) in it.enumerate() { print!("{}:{} ", i, x); } println!(); }
fn wl(it: impl Iterator<Item = char>) -> String { let mut p = it.peekable(); let mut s = String::new(); while let Some(c) = p.next() { s.push(c); if let Some(&n) = p.peek() { if n == 'b' { s.push('!'); } } } s }
fn by_ref(mut it: impl Iterator<Item = i32>) -> (Option<i32>, i32) { let a = it.next(); (a, it.sum()) }
fn twice<T: std::fmt::Debug>(it: impl Iterator<Item = T>) -> Vec<String> { it.map(|x| format!("{:?}{:?}", x, x)).collect() }
fn main() {
    println!("{}", outer(vec![1, 2].into_iter()));
    loop_it(vec![5i8, 6].into_iter());
    println!("{}", wl("abcab".chars()));
    println!("{:?}", by_ref(0..4));
    println!("{:?}", twice(vec![Some(1)].into_iter()));
    println!("{:?}", twice("xy".chars()));
}

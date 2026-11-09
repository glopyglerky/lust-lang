// expect: L0306
// rustc E0597: a where-clause bound's `'static` output ties the closure's result.
fn run<F>(f: F) -> usize
where
    F: Fn() -> &'static [u8],
{
    f().len()
}

fn main() {
    let v = vec![1u8, 2];
    println!("{}", run(|| &v[..]));
}

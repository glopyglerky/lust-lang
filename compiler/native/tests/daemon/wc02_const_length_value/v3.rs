// A named const array length in a written type changes under an unchanged body (no error,
// other output): the body must check and generate again (fn-body read sets, 2026-10-20).
const N: usize = 1;

fn f() -> usize {
    let a: [u8; N] = std::array::from_fn(|i| i as u8 + 3);
    a.len() * 100 + a.iter().map(|&x| x as usize).sum::<usize>()
}

fn main() {
    println!("{}", f());
}

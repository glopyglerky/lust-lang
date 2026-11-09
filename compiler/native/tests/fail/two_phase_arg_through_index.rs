// expect: L0303
// rustc E0502: `v[0]` reborrows through `IndexMut`, which is not two-phase.
fn f(c: &mut u32, k: usize) { *c += k as u32; }
fn main() {
    let mut a = 1u32; let mut b = 2u32;
    let mut v: Vec<&mut u32> = vec![&mut a, &mut b];
    f(v[0], v.len());
    println!("{}", v.len());
}

// A generic callable passed on by reference: core's `impl Fn* for &F` and
// `impl FnMut/FnOnce for &mut F` (ts-rust ast/utilities_p2.rs
// find_ancestor_with_kind, lsp/server.rs run_idle_work, printer/emit_context.rs
// for_each_value_node).
fn walk(n: u32, mut cb: impl FnMut(u32, u8) -> bool) -> u32 { if cb(n, 1) { n } else { 0 } }
fn find(n: u32, mut callback: impl FnMut(u32, u8) -> bool) -> u32 { walk(n, &mut callback) + walk(n + 1, &mut callback) }
fn drop_garbage(busy: &impl Fn() -> bool) -> bool { busy() }
fn idle(busy: impl Fn() -> bool) -> bool { drop_garbage(&busy) }
fn each(mut f: impl FnMut(u32)) { for x in [1u32, 2] { f(x); } [3u32, 4].iter().copied().for_each(&mut f); vec![5u32].into_iter().for_each(&mut f); }
fn count<F: Fn(&u32) -> bool>(f: &F) -> usize { [1u32, 2, 3, 4].iter().copied().filter(f).count() }
fn main() {
    let mut seen = Vec::new();
    println!("{}", find(3, |a, b| { seen.push(a); a % 2 == b as u32 % 2 }));
    println!("{:?} {}", seen, idle(|| true));
    let mut acc = 0;
    each(|x| acc += x);
    println!("{} {}", acc, count(&|x: &u32| *x > 2));
}

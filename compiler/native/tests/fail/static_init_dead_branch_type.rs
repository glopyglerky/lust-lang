// expect: L0110
// rustc E0308: as const_init_dead_branch_type, for a static.
static N: u32 = if true { 1 } else { 'x' };
fn main() {
    println!("{}", N);
}

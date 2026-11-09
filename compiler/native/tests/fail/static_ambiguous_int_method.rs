// expect: L0111
static C: usize = 1.wrapping_add(3);

fn main() {
    println!("{}", C);
}

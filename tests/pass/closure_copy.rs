// A closure whose captures are all Copy is Copy itself.
fn run<F: Fn() -> u32>(f: F) -> u32 {
    f()
}
fn main() {
    let x = 5u32;
    let c = || x * 2;
    println!("{} {}", run(c), run(c));
    let y = 6u32;
    let m = move || y + 1;
    let m2 = m;
    println!("{} {}", m(), m2());
}

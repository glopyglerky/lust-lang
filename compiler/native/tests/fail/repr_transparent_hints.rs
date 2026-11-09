// expect: L0110
#[repr(transparent, C)]
struct S(u32);

fn main() {
    let _s = S(1);
    println!("x");
}

// capability: rejected: `unsafe` blocks and functions
// expect: L0404

fn main() {
    let x = 1;
    let p = &x as *const i32;
    println!("{}", unsafe { *p });
}

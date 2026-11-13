// capability: rejected: raw references (`&raw`)
// expect: L0820

fn main() {
    let x = 1;
    let p = &raw const x;
    println!("{}", p.is_null());
}

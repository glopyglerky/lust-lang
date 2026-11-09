// expect: L0631
// rustc E0493: a destructor in an anonymous const is an error, also in a dead branch.
#[repr(u8)]
enum K {
    A = {
        if false {
            let _s = String::new();
        }
        3
    },
}
fn main() {
    println!("{}", K::A as u8);
}

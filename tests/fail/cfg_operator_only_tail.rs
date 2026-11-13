// expect: L0001
// rustc E0658: the same with the tail first in its block.
fn main() {
    let mut k = 0;
    let x = {
        #[cfg(all())]
        k += 1
    };
    println!("{:?} {k}", x);
}

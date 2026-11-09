// expect: L0001
// rustc E0658: a cfg on a block tail whose top level is a binary operation
// sits on the left operand; removing it is an error, not a dropped tail.
fn t() -> i32 { 1 }
fn main() {
    let x = {
        let a = 2;
        #[cfg(any())]
        a - t()
    };
    println!("{:?}", x);
}

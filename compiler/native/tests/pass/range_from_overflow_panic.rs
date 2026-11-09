// `next` at u8::MAX steps past it first: core's Step::forward panics at
// the step_integer_impls! line before 255 comes out.
fn main() {
    let mut r = 254u8..;
    println!("{:?}", r.next());
    println!("{:?}", r.next());
}

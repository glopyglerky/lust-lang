// A panic in tokens a macro_rules body writes reports at the macro call,
// also when the expression mixes the macro's tokens with its argument.
// panic: attempt to add with overflow
macro_rules! at {
    ($v:expr, $i:expr) => {
        $v[$i + 1]
    };
}
macro_rules! bump {
    ($x:expr) => {
        $x + 251u8
    };
}
fn main() {
    let n = std::env::args().count();
    let v = vec![1u8, 2, 3];
    println!("{}", at!(v, n));
    let o = Some(4u8);
    println!("{}", bump!(o.unwrap() + n as u8));
}

#[derive(Clone)]
struct C;
const fn take(_c: C) -> usize {
    1
}
struct P {
    a: [u8; {
        if false {
            let s = C;
            let _t = take(s);
            let _u = take(s);
        }
        3
    }],
}
fn main() {
    println!("{}", std::mem::size_of::<P>());
}

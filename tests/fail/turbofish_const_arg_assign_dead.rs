// expect: L0302
// rustc E0384: a turbofish const argument in a body is checked as code.
fn f<const N: usize>() -> usize {
    N
}
fn main() {
    println!("{}", f::<{
        if false {
            let n = 1;
            n = 2;
        }
        2
    }>());
}

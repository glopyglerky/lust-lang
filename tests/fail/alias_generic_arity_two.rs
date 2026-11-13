// expect: L0110
// rustc E0107: the alias takes one argument.
type Twice<T> = (T, T);

fn main() {
    let t: Twice<u8, u8> = (1, 2);
    println!("{}", t.0);
}

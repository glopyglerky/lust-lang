// expect: L0110
// rustc E0308: `Twice<u8>` is `(u8, u8)`; a `&str` is a mismatch.
type Twice<T> = (T, T);

fn main() {
    let t: Twice<u8> = (1, "two");
    println!("{}", t.0);
}

// Generic aliases in const and static types, with args and defaults.
type Twice<T> = (T, T);
const X: Twice<u8> = (1, 2);
static Y: Twice<u16> = (3, 4);
fn main() {
    println!("{} {} {}", X.0, Y.1, P.1);
}

type Pair<T = u8> = (T, T);
const P: Pair = (5, 6);

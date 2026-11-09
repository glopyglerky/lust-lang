// Primitive associated consts through a type alias and the qualified
// form (ts-rust `const BIG_ID: IdCell = IdCell::MAX`).
type IdCell = u32;
const BIG_ID: IdCell = IdCell::MAX;
type Small = i8;
const LO: Small = Small::MIN;
const B: u64 = <u64>::MAX - 1;
fn main() {
    println!("{} {} {}", BIG_ID, LO, B);
}

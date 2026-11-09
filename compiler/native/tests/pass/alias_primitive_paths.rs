// Type-relative paths through aliases of primitives, tuples, and arrays
// (`U::MAX`, `Rune::from(c)`, `Pair::default()`), and std trait fns on
// builtin types by path (`String::default()`, `Vec::<u8>::default()`).
type U = u8;
type Rune = u32;
type Wide = i64;
type Pair = (u8, String);
type Arr = [i16; 3];
type C4 = Vec<char>;

fn main() {
    println!("{} {} {}", U::MAX, U::MIN, U::BITS);
    println!("{} {}", Rune::from('a'), Wide::from(-3i32));
    let p = Pair::default();
    let a = Arr::default();
    println!("{:?} {:?}", p, a);
    println!("{:?} {:?} {:?}", String::default(), Vec::<u8>::default(), C4::default());
    println!("{:?} {:?}", Option::<u8>::default(), std::vec::Vec::<u8>::new::<>());
}

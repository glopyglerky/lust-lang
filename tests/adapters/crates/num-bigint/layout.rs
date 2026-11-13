// num-bigint 0.4.6 layouts lust answers: Sign is listed exact-layout in
// the adapter manifest (a fieldless enum, as upstream), so size_of and
// align_of agree with the real crate, also through Option.
use num_bigint::Sign;

fn main() {
    println!("{} {} {}", std::mem::size_of::<Sign>(), std::mem::align_of::<Sign>(), std::mem::size_of::<Option<Sign>>());
    println!("{}", std::mem::size_of::<&num_bigint::BigInt>());
}

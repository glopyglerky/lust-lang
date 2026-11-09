// lust-reject: L0652
// The adapter's Xxh3 keeps its input in a Vec, not upstream's fixed
// buffer, so lust must not answer size_of for it (ARCH 3.6).
fn main() {
    println!("{}", std::mem::size_of::<xxhash_rust::xxh3::Xxh3>() > 0);
}

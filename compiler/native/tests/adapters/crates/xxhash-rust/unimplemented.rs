// lust-reject: L0506
// The `_with_secret` forms are not in the adapter, so the import rejects.
use xxhash_rust::xxh3::xxh3_64_with_secret;

fn main() {
    let secret = [7u8; 192];
    println!("{:016x}", xxh3_64_with_secret(b"abc", &secret));
}

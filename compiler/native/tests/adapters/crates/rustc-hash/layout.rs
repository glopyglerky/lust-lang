// rustc-hash 2.1.2 layouts lust answers: FxHasher, FxBuildHasher, and
// FxSeededState are exact-layout in the adapter manifest.
use rustc_hash::{FxBuildHasher, FxHasher, FxSeededState};

fn main() {
    println!("{} {} {}", std::mem::size_of::<FxHasher>(), std::mem::size_of::<FxBuildHasher>(), std::mem::size_of::<FxSeededState>());
    println!("{} {}", std::mem::size_of::<Option<FxHasher>>(), std::mem::align_of::<FxHasher>());
}

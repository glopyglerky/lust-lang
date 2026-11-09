// rustix 1.1.5 trait forms as upstream has them (fx-splice-derive):
// `Errno` hashes its raw negated `u16`, `FileType` discriminants are the
// `S_IF*` bits.
use rustix::fs::FileType;
use rustix::io::Errno;
use std::hash::{Hash, Hasher};

fn main() {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    Errno::NOENT.hash(&mut h);
    println!("{}", h.finish());
    println!("{} {} {}", FileType::RegularFile as isize, FileType::Directory as isize, FileType::Unknown as isize);
}

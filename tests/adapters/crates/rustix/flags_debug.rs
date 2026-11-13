// lust-reject: L0112
// Upstream prints a flags value in the bitflags form (`OFlags(RDWR)`);
// the adapter has no such Debug, so a use rejects.
use rustix::fs::OFlags;

fn main() {
    println!("{:?}", OFlags::RDWR);
}

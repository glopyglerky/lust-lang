// ARCH 2.3 and 2.6, the orphan rule's accepted sides in one file:
// a local trait for a foreign type, and a foreign trait for a
// local type. The rejected side is fail/orphan_foreign_foreign.rs.
trait Local {
    fn v(&self) -> u8;
}
impl Local for u32 {
    fn v(&self) -> u8 {
        3
    }
}
struct Mine;
impl std::fmt::Display for Mine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("mine")
    }
}
fn main() {
    println!("{} {}", 7u32.v(), Mine);
}

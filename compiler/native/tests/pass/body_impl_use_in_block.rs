// A body impl of a trait a block use brings in (ts-rust go_frontend.rs OsHost).
mod tr { pub trait Host { fn cwd(&self) -> &str; } pub fn run(h: &dyn Host) -> String { h.cwd().to_uppercase() } }
struct Thing { d: String }
impl Thing {
    fn finish(self) -> String {
        use crate::tr::{Host, run};
        struct OsHost { dir: String }
        impl Host for OsHost { fn cwd(&self) -> &str { &self.dir } }
        run(&OsHost { dir: self.d })
    }
}
fn main() { println!("{}", Thing { d: "abc".into() }.finish()); }

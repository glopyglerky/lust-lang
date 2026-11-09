// expect: L0112
// rustc E0277: `Arena`'s derived Debug needs `CowChunks<Table>: Debug`,
// which the derive on `CowChunks<T>` gives only under `T: Debug`, and
// `Table` has no Debug. Checked at the definition, with no use of
// `{:?}` (ts-rust core.rs, SymbolArena over CowChunks<Table>).
#[derive(Debug)]
enum Chunk<T: 'static> {
    Shared(&'static [T]),
    Freed,
}
#[derive(Debug)]
struct CowChunks<T: 'static> {
    chunks: Vec<Chunk<T>>,
    tail: Vec<T>,
}
struct Table {
    n: u8,
}
#[derive(Debug)]
struct Arena {
    tables: CowChunks<Table>,
}
fn main() {
    let a = Arena { tables: CowChunks { chunks: vec![Chunk::Freed], tail: vec![Table { n: 3 }] } };
    println!("{} {}", a.tables.tail[0].n, a.tables.chunks.len());
}

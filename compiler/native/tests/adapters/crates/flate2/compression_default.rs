// flate2 1.1.10: the default level is 6 (fx-splice-derive).
use flate2::Compression;

fn main() {
    println!("{} {:?}", Compression::default().level(), Compression::default());
}

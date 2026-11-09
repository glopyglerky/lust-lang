#[derive(Debug, Clone, Copy)]
enum E {
    V2 { x: u8 },
    V0,
    V1(i64),
}
fn main() {
    let es = [E::V0, E::V1(4), E::V2 { x: 9 }];
    println!("{}", es.len());
}

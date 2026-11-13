#[derive(Debug, Clone, Copy)]
enum E {
    V0,
    V1(i64),
    V2 { x: u8 },
}
fn main() {
    let es = [E::V0, E::V1(4), E::V2 { x: 9 }];
    println!("{}", es.len());
    println!("{:?}", es[1]);
}

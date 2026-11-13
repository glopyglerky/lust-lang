// An Adt instance reached only through derived Default glue gets its
// field instances too (Out<u8> holds In<u8>; it printed In(())).
#[derive(Debug, Default, Clone, Copy)]
struct In<T>(T);
#[derive(Debug, Default, Clone, Copy)]
struct Out<T> { value: In<T> }
fn main() {
    let m: std::mem::ManuallyDrop<(u8, Option<i32>)> = Default::default();
    println!("{:?}", m);
    let c: Out<u8> = Default::default();
    println!("{:?}", c);
    let d: Out<(u8, Option<i32>)> = Out::default();
    println!("{:?}", d);
    let e = Out::<String>::default();
    println!("{:?}", e);
}

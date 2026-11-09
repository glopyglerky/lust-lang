// Deref coercion through an inner `&mut` (ts-rust ArenaBox Deref): `&&mut T`
// to `&T` and `&mut &mut T` to `&mut T` reborrow the pointee.
enum R<T: 'static> {
    A(&'static mut T),
    B(Box<T>),
}
fn get<T>(r: &R<T>) -> &T {
    match r {
        R::A(v) => v,
        R::B(v) => v,
    }
}
fn get_mut<T>(r: &mut R<T>) -> &mut T {
    match r {
        R::A(v) => v,
        R::B(v) => v,
    }
}
fn shared<'a, 'b>(x: &'a &'b mut Vec<i32>) -> &'a Vec<i32> {
    x
}
fn deep<'a>(x: &'a &'a &'a mut i32) -> &'a i32 {
    x
}
fn main() {
    let mut r = R::B(Box::new(5));
    *get_mut(&mut r) += 1;
    println!("{}", get(&r));
    let mut v = vec![1, 2];
    let m = &mut v;
    println!("{:?}", shared(&m));
    m.push(3);
    let mut n = 7;
    let k = &mut n;
    let kk = &k;
    println!("{}", deep(&kk));
    println!("{:?}", v);
}

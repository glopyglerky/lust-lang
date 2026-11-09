// Floats keep `PartialEq` and `PartialOrd` everywhere: bounds on
// float tuples, arrays, slices, and containers, the operators,
// `partial_cmp`, `sort_by`, and `dedup`. `PhantomData<f32>`,
// raw pointers to floats, and `Duration` in a tuple are `Eq`.
fn pe<T: PartialEq>(a: &T, b: &T) -> bool { a == b }
fn po<T: PartialOrd>(a: &T, b: &T) -> bool { a < b }
fn eq<T: Eq>(_: &T) -> bool { true }

fn main() {
    let t = (1.5f32, 2.5f64);
    println!("{} {}", pe(&t, &t), po(&t, &(9.5f32, 0.5f64)));
    let a = [1.5f32, 2.5f32];
    println!("{} {:?}", pe(&a, &a), a.partial_cmp(&[9.5f32, 0.5f32]));
    let v = vec![(1u8, 1.5f32)];
    println!("{}", pe(&v, &v));
    assert_eq!((1.5f32, 2.5f32), (1.5f32, 2.5f32));
    let mut s = vec![2.5f32, 1.5f32, 1.5f32];
    s.sort_by(|x, y| x.partial_cmp(y).unwrap());
    s.dedup();
    println!("{} {}", s.len(), s[0]);
    let ph: (std::marker::PhantomData<f32>, u8) = (std::marker::PhantomData, 1u8);
    println!("{}", eq(&ph));
    let x = 1.5f32;
    println!("{}", eq(&(&x as *const f32, 1u8)));
    println!("{}", eq(&(std::time::Duration::from_secs(1), 1u8)));
}

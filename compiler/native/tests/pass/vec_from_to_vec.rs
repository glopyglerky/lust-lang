// Vec::from in its slice (clone, exact capacity) and array
// (move) forms, slice and array to_vec, and the generic forms
// of the Clone/PartialEq-requiring methods.
fn ext<T: Clone>(v: &mut Vec<T>, s: &[T]) {
    v.extend_from_slice(s);
}
fn dd<T: PartialEq>(v: &mut Vec<T>) {
    v.dedup();
}
fn main() {
    let a = [1, 2, 3];
    let v = a.to_vec();
    println!("{:?} {}", v, v.capacity());
    let s: &[i32] = &a;
    println!("{:?}", s.to_vec());
    let x: Vec<i32> = Vec::from([4, 5]);
    println!("{:?} {}", x, x.capacity());
    let y: Vec<i32> = Vec::from(&a[..]);
    println!("{:?} {}", y, y.capacity());
    let z: Vec<i32> = Vec::from(&a);
    println!("{:?}", z);
    let words = vec!["hi".to_string()];
    let w2 = words.as_slice().to_vec();
    println!("{:?}", w2);
    let owned: Vec<String> = Vec::from(["a".to_string(), "b".to_string()]);
    println!("{:?}", owned);
    let mut g = vec![1, 2];
    ext(&mut g, &[3, 4]);
    println!("{:?}", g);
    let mut d = vec![1, 1, 2];
    dd(&mut d);
    println!("{:?}", d);
    let mut nested: Vec<Vec<i32>> = Vec::new();
    nested.resize(2, vec![1, 2]);
    nested[0].push(3);
    println!("{:?}", nested);
}

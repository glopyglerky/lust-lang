// Slice-to-array `TryFrom`/`try_into` (tf-slice): by value (Copy),
// by shared and mutable reference, in method, associated, and
// qualified forms, with `TryFromSliceError` Debug and Display.
fn by_val<T: Copy>(s: &[T]) -> [T; 2] {
    s.try_into().unwrap()
}
fn by_ref<T>(s: &[T]) -> &[T; 2] {
    s.try_into().unwrap()
}
fn main() {
    let v = vec![1u8, 2, 3];
    let a: [u8; 3] = v.as_slice().try_into().unwrap();
    println!("{:?}", a);
    let r: &[u8; 2] = v[..2].try_into().unwrap();
    println!("{:?}", r);
    let bad: Result<[u8; 2], _> = v.as_slice().try_into();
    println!("{:?} {}", bad, bad.unwrap_err());
    let b2: Result<&[u8; 9], _> = v[..].try_into();
    println!("{}", b2.unwrap_err());
    println!("{:?}", <[u8; 2]>::try_from(&v[..2]).unwrap());
    println!("{:?}", <&[u8; 2]>::try_from(&v[..2]).unwrap());
    let q: [u8; 2] = <[u8; 2] as TryFrom<&[u8]>>::try_from(&v[1..]).unwrap();
    println!("{:?}", q);
    println!("{:?} {:?}", by_val(&v[..2]), by_ref(&v[..2]));
    let mut m = vec![4u8, 5, 6];
    let mr: &mut [u8; 2] = (&mut m[..2]).try_into().unwrap();
    mr[0] = 40;
    println!("{:?}", m);
    let mv: [u8; 2] = (&mut m[1..]).try_into().unwrap();
    println!("{:?}", mv);
    let x: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    for (a, b) in x.chunks_exact(4).zip(x.chunks_exact(4)) {
        let p: [u8; 4] = a.try_into().unwrap();
        let q: &[u8; 4] = b.try_into().unwrap();
        println!("{:?} {:?}", p, q);
    }
    let e: &[u8] = b"";
    let z: [u8; 0] = e.try_into().unwrap();
    println!("{:?}", z);
    let s = "ab";
    let sa: [u8; 2] = s.as_bytes().try_into().unwrap();
    println!("{:?}", sa);
    let w: [String; 1] = ["x".to_string()];
    let wr: &[String; 1] = w[..].try_into().unwrap();
    println!("{:?}", wr);
}

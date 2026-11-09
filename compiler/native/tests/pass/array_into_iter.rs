// array::IntoIter: random access in a zip (a mapped side is pulled only
// min times), TrustedLen collect capacity, String extend from arrays and
// repeat_n.
fn main() {
    let z: Vec<(i32, i32)> = [1, 2, 3].into_iter().map(|x| { println!("a{}", x); x }).zip([10].into_iter()).collect();
    println!("{:?}", z);
    let z2: Vec<(i32, i32)> = [10].into_iter().zip([1, 2, 3].into_iter().map(|x| { println!("b{}", x); x })).collect();
    println!("{:?}", z2);
    let v: Vec<char> = ['a', 'b'].into_iter().collect();
    println!("{} {}", v.capacity(), [1, 2, 3].into_iter().count());
    let mut s = String::new();
    s.extend(['x'; 3]);
    s.extend(std::iter::repeat_n('y', 2));
    println!("{} {}", s, s.capacity());
    let mut w: Vec<u8> = Vec::new();
    w.extend([1u8, 2]);
    w.extend(std::iter::repeat_n(7u8, 3));
    w.extend(&[9u8, 9]);
    println!("{:?} {}", w, w.capacity());
}

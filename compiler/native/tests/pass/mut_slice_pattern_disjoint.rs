fn main() {
    let mut v = vec![1, 2, 3, 4];
    if let [first, rest @ ..] = v.as_mut_slice() {
        *first = 10;
        for r in rest.iter_mut() {
            *r *= 2;
        }
    }
    println!("{:?}", v);
    if let [.., a, b] = &mut v[..] {
        std::mem::swap(a, b);
    }
    println!("{:?}", v);
    if let [a, .., b] = v.as_mut_slice() {
        *a += 1;
        *b += 1;
    }
    println!("{:?}", v);
    if let [a, mid @ .., b] = v.as_mut_slice() {
        *a += 1;
        mid[0] += 1;
        *b += 1;
    }
    println!("{:?}", v);
    if let [rest @ .., last] = v.as_mut_slice() {
        rest[0] += 1;
        *last += 1;
    }
    println!("{:?}", v);
    let mut w = [5, 6, 7, 8];
    if let [first, rest @ ..] = &mut w[..] {
        *first = 50;
        rest[2] = 80;
    }
    println!("{:?}", w);
    match v.as_mut_slice() {
        [first, .., last] => {
            *first += 100;
            *last += 100;
        }
        _ => {}
    }
    println!("{:?}", v);
}

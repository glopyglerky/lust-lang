// Vec capacity must follow Rust's RawVec growth byte for byte
// (ARCH 3.3/3.5): minimum capacity by element size, doubling on
// the amortized path, exact on reserve_exact and with_capacity.
fn caps<T>(name: &str, v: &Vec<T>) {
    println!("{} cap {} len {}", name, v.capacity(), v.len());
}
fn main() {
    let mut a: Vec<u8> = Vec::new();
    for i in 0..10 {
        a.push(i as u8);
        caps("u8", &a);
    }
    let mut b: Vec<u32> = Vec::new();
    for i in 0..10 {
        b.push(i);
        if i < 5 || i == 9 {
            caps("u32", &b);
        }
    }
    let mut c: Vec<[u8; 2000]> = Vec::new();
    caps("big0", &c);
    c.push([0; 2000]);
    caps("big1", &c);
    c.push([0; 2000]);
    caps("big2", &c);
    let mut d: Vec<u8> = Vec::new();
    d.reserve(3);
    caps("r3", &d);
    d.reserve_exact(100);
    caps("re100", &d);
    d.extend_from_slice(&[1, 2]);
    caps("ext", &d);
    d.shrink_to(50);
    caps("sh50", &d);
    d.shrink_to_fit();
    caps("shfit", &d);
    let e = d.clone();
    caps("clone", &e);
    let mut v: Vec<i32> = Vec::new();
    v.push(1);
    v.reserve(v.len());
    caps("self-reserve", &v);
    v.shrink_to(0);
    caps("sh0", &v);
    let z: Vec<()> = Vec::new();
    println!("zst {}", z.capacity() == usize::MAX);
    let w = vec![1i32, 2, 3];
    caps("vec!", &w);
    let x: Vec<i32> = Vec::with_capacity(7);
    caps("withcap", &x);
}

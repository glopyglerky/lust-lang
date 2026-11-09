fn seq<T: Default + Clone>(name: &str, make: impl Fn() -> T) {
    let mut v: Vec<T> = Vec::new();
    print!("{name} push:");
    for _ in 0..20 { v.push(make()); print!(" {}", v.capacity()); }
    println!();
    let mut v: Vec<T> = Vec::new();
    print!("{name} reserve:");
    for n in [1usize,2,3,5,10,100] { v.reserve(n); print!(" r{n}={}", v.capacity()); }
    println!();
    let mut v: Vec<T> = Vec::new();
    print!("{name} reserve_exact:");
    for n in [1usize,2,3,5,10,100] { v.reserve_exact(n); print!(" r{n}={}", v.capacity()); }
    println!();
    let mut v: Vec<T> = Vec::new();
    for _ in 0..10 { v.push(make()); }
    print!("{name} shrink: start={}", v.capacity());
    v.shrink_to(12); print!(" to12={}", v.capacity());
    v.shrink_to(10); print!(" to10={}", v.capacity());
    v.truncate(3);
    v.shrink_to(5); print!(" len3to5={}", v.capacity());
    v.shrink_to_fit(); print!(" fit={}", v.capacity());
    v.clear(); v.shrink_to_fit(); print!(" emptyfit={}", v.capacity());
    println!();
}
#[derive(Clone)] struct B1024([u8; 1024]); impl Default for B1024 { fn default() -> Self { B1024([0; 1024]) } }
#[derive(Clone)] struct B1025([u8; 1025]); impl Default for B1025 { fn default() -> Self { B1025([0; 1025]) } }
fn main() {
    seq("u8", || 0u8); seq("u16", || 0u16); seq("u32", || 0u32); seq("u64", || 0u64);
    seq("u128", || 0u128); seq("b1024", B1024::default); seq("b1025", B1025::default);
    seq("zst", || ());
    // String growth
    let mut s = String::new();
    print!("String push_str:");
    for n in [1usize,7,1,8,100,1000] { s.push_str(&"x".repeat(n)); print!(" +{n}={}", s.capacity()); }
    println!();
    let mut s = String::new();
    s.reserve(3); print!("String reserve3={}", s.capacity());
    s.reserve_exact(3); println!(" reserve_exact3={}", s.capacity());
    // with_capacity exactness
    println!("with_cap u8: {} {} {}", Vec::<u8>::with_capacity(0).capacity(), Vec::<u8>::with_capacity(3).capacity(), Vec::<u8>::with_capacity(100).capacity());
    // collect capacity (trusted len)
    let v: Vec<u8> = (0..34u8).collect(); println!("collect34 u8 cap={} len={}", v.capacity(), v.len());
    let v: Vec<u8> = (0..100u8).filter(|x| x % 3 == 0).collect(); println!("filter collect cap={} len={}", v.capacity(), v.len());
}

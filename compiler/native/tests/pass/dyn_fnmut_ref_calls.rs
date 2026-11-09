// `&mut dyn FnMut` values: forwarded between fns, called in a loop,
// mutating captures the caller reads afterwards.
fn collect(f: &mut dyn FnMut(i32) -> i32) -> i32 {
    f(1) + f(2)
}
fn outer(f: &mut dyn FnMut(i32) -> i32) -> i32 {
    collect(f)
}
fn drive(f: &mut dyn FnMut(i32)) {
    for i in 0..4 {
        f(i);
    }
}
fn count(f: &mut dyn FnMut() -> u32) -> u32 {
    f();
    f();
    f()
}
fn main() {
    let mut add = |n: i32| n + 100;
    println!("{}", outer(&mut add));
    let mut sum = 0;
    let mut log: Vec<i32> = Vec::new();
    drive(&mut |n| {
        sum += n;
        log.push(n * 10);
    });
    println!("{} {:?}", sum, log);
    let mut c = 0u32;
    let mut inc = || {
        c += 1;
        c
    };
    println!("{}", count(&mut inc));
    println!("{}", c);
}

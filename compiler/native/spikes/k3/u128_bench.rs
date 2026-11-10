// K3: rustc native u128 timing, same shape as u128_bench.c.
use std::hint::black_box;
use std::time::Instant;
fn main() {
    let iters: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(2_000_000);
    let mut state = 0x853c_49e6_748f_ea9bu64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut a = [0u128; 4096];
    let mut b = [0u128; 4096];
    for i in 0..4096 {
        a[i] = ((next() as u128) << 64) | next() as u128;
        b[i] = ((next() as u128) << 64) | (next() | 1) as u128;
    }
    let run = |name: &str, n: u64, f: &dyn Fn(u128, u128, u64) -> u128| {
        let t = Instant::now();
        let mut sink: u128 = 0;
        for i in 0..n {
            sink = sink.wrapping_add(f(a[(i & 4095) as usize], b[(i & 4095) as usize], i));
        }
        black_box(sink);
        println!("BENCH rust_u128_{} ns_per_op={:.2} iters={}", name, t.elapsed().as_nanos() as f64 / n as f64, n);
    };
    run("add", iters, &|x, y, _| x.wrapping_add(y));
    run("mul", iters, &|x, y, _| x.wrapping_mul(y));
    run("div", iters / 20 + 1, &|x, y, _| x / y);
    run("shl", iters, &|x, _, i| x.wrapping_shl((i & 127) as u32));
    run("format", iters / 20 + 1, &|x, _, _| x.to_string().len() as u128);
    run("cmp", iters, &|x, y, _| (x >= y) as u128);
}

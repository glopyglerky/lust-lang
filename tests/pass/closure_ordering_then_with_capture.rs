// A closure body chains Ordering methods on a then_with whose closure reads the
// outer closure's capture (ts-rust api/session_p2.rs sort_func comparator).
fn sort_func<T, F: FnMut(&T, &T) -> i32>(x: &mut [T], mut cmp: F) {
    let n = x.len();
    for i in 0..n { for j in 0..n - 1 - i { if cmp(&x[j], &x[j + 1]) > 0 { x.swap(j, j + 1); } } }
}
struct Syms { names: Vec<&'static str> }
fn name(s: &Syms, i: u32) -> &'static str { s.names[i as usize] }
enum R { File { symbols: Syms, file: u32 } }
fn main() {
    let r = R::File { symbols: Syms { names: vec!["b", "a", "c"] }, file: 7 };
    let mut subs = vec![0u32, 1, 2];
    match &r {
        R::File { symbols, file } => {
            let file = *file;
            sort_func(&mut subs, |&left, &right| {
                assert!(file == 7);
                if left == right { return 0; }
                let order = std::cmp::Ordering::Equal
                    .then_with(|| name(symbols, left).cmp(&name(symbols, right)))
                    .then_with(|| left.cmp(&right));
                order as i32
            });
        }
    }
    println!("{:?}", subs);
}

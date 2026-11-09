// tikv-jemallocator 0.7.0 as ts-rust's bin roots write it: the
// #[global_allocator] static, then allocation-heavy work whose
// output must not depend on the allocator.
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

use std::collections::HashMap;

fn main() {
    let mut v: Vec<String> = Vec::new();
    for i in 0..200 {
        v.push(format!("item-{i}-{}", i * i % 97));
    }
    v.sort();
    let mut m: HashMap<String, usize> = HashMap::new();
    for (i, s) in v.iter().enumerate() {
        m.insert(s.clone(), i);
    }
    let mut keys: Vec<&String> = m.keys().collect();
    keys.sort();
    println!("{} {} {} {}", v.len(), m.len(), keys[0], keys[keys.len() - 1]);
    let total: usize = v.iter().map(|s| s.len()).sum();
    println!("{total} {:?}", GLOBAL);
    let boxed: Vec<u64> = (0..1000u64).map(|i| i ^ (i << 3)).collect();
    println!("{}", boxed.iter().sum::<u64>());
}

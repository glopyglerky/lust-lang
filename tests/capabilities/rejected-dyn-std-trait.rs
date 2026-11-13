// capability: rejected: `dyn` with a std trait other than the Fn family, `Any`, `Error`, `Debug` and `Display` (`dyn Iterator`, `dyn Hasher`)
// expect: L0702

fn total(it: &mut dyn Iterator<Item = u32>) -> u32 {
    it.sum()
}

fn main() {
    println!("{}", total(&mut vec![1u32, 2].into_iter()));
}

// Iterator adapters on `I: Iterator` and `impl Iterator` parameters and
// on `<I as IntoIterator>::IntoIter` (W-STD-B): the template types them
// through the bound, each instance runs its concrete iterator.
fn bump<I: Iterator<Item = u8>>(it: I) -> Vec<u8> {
    it.map(|x| x + 1).collect()
}

fn add(it: impl Iterator<Item = u8>) -> u8 {
    it.sum()
}

fn show<I>(it: I) -> usize
where
    I: Iterator,
    I::Item: std::fmt::Debug,
{
    it.map(|x| format!("{x:?}")).count()
}

fn sizes<I: IntoIterator<Item = u8>>(it: I) -> usize {
    it.into_iter().filter(|x| *x > 1).count()
}

fn all_from<I: IntoIterator<Item = u8>>(it: I) -> u32 {
    let mut s = 0;
    for x in it {
        s += x as u32;
    }
    s
}

fn main() {
    println!("{:?} {:?}", bump(vec![1u8, 2].into_iter()), bump([7u8].into_iter()));
    println!("{} {}", add([1u8, 2].into_iter()), add(vec![3u8].into_iter().map(|x| x * 2)));
    println!("{}", show(["a", "b"].into_iter()));
    println!("{} {}", sizes(vec![1u8, 2, 3]), all_from(vec![4u8, 5]));
    let v = vec![9u8, 1];
    println!("{} {}", sizes(v.iter().copied().collect::<Vec<u8>>()), all_from(v));
}

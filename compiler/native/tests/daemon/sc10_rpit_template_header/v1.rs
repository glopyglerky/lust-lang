pub struct Page<V> {
    pub v: Vec<V>,
}

impl<V: Default + Clone> Page<V> {
    const FIRST: usize = 64 / (std::mem::size_of::<V>() - 1);
    pub fn new() -> Self {
        Page { v: vec![V::default(); Self::FIRST % 5 + 1] }
    }
}

pub fn odd_indexes<V: Clone>(v: &[V], k: u32) -> impl Iterator<Item = usize> + '_  {
    (0..v.len()).filter(move |i| i % 2 == 1 && (*i as u64) < k as u64 + 10)
}

pub fn use_page<V: Default + Clone>() -> usize {
    let p: Page<V> = Page::new();
    odd_indexes(&p.v, 3).count()
}

fn main() {
    let xs = vec![1u16, 2, 3, 4];
    println!("{} {} {}", odd_indexes(&xs, 2).count(), use_page::<u32>(), use_page::<u16>());
}

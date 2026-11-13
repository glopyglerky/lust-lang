pub struct Page<V> {
    pub v: Vec<V>,
}

impl<V: Default + Clone> Page<V> {
    const FIRST: usize = 64 / (std::mem::size_of::<V>() - 1);
    pub fn new() -> Self {
        Page { v: vec![V::default(); Self::FIRST % 5 + 1] }
    }
}

fn mk<T: Default + Clone>(_t: T) -> usize {
    Page::<T>::new().v.len()
}

fn tc<V: Default + Clone>(_v: V) -> usize {
    mk(1u8)
}

fn main() {
    println!("{}", tc(3u16));
}

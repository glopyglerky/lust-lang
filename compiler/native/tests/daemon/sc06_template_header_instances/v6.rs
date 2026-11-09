pub struct Page<V> {
    pub v: Vec<V>,
}

impl<V: Default + Clone> Page<V> {
    const FIRST: usize = 64 / (std::mem::size_of::<V>() - 1);
    pub fn new() -> Self {
        Page { v: vec![V::default(); Self::FIRST % 5 + 1] }
    }
}

pub fn mk<T: Default + Clone>(_t: T) -> usize {
    Page::<T>::new().v.len()
}

pub fn tw<V: Default + Clone>(v: V, k: u32) -> usize  {
    mk(v) + k as usize
}

fn main() {
    println!("{} {}", tw(3u16, 2), tw(1u32, 3));
}

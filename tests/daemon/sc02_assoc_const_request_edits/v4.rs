pub struct W<T>(pub T);

impl<T> W<T> {
    const N: usize = 1000 / (std::mem::size_of::<T>() - 1);
    pub fn n() -> usize {
        Self::N
    }
}

fn g<T>() -> usize {
    W::<T>::n()
}

fn main() {
    println!("{}", g::<u16>());
}

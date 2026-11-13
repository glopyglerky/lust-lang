// A generic param shadows module items of the same name for a path's
// first segment, as in rustc. The resolver bound such a param to the item:
// in a non-root module, `size_of::<T>()` measured `struct T` (wrong output),
// and a param named like an item of another module was "cannot find type".
// A type param is not a value, so the value `T` below is the static.
mod m {
    pub struct T(pub [u8; 32]);
    pub const N: usize = 9;
    pub static S: u8 = 2;

    pub fn size<T>() -> usize {
        std::mem::size_of::<T>()
    }

    pub struct W<T>(pub T);

    impl<T: Copy> W<T> {
        pub fn get(&self) -> T {
            self.0
        }
    }

    pub fn count<const N: usize>() -> usize {
        N
    }

    pub fn zeros<const N: usize>() -> [u8; N] {
        [0; N]
    }

    pub fn make<N: Default>() -> N {
        N::default()
    }

    pub fn static_value<S>(_s: S) -> u8 {
        S
    }
}

mod a {
    pub struct K;
}

mod b {
    pub type Two<K> = (K, K);

    pub fn first<K: Copy>(t: Two<K>) -> K {
        t.0
    }
}

pub use b::Two;

fn main() {
    let t = m::T([1; 32]);
    let _k = a::K;
    let pair: Two<u16> = (5, 6);
    println!("{} {} {}", m::size::<u8>(), m::size::<u64>(), t.0.len());
    println!("{} {}", m::W(7u8).get(), m::W('x').get());
    println!("{} {} {}", m::count::<4>(), m::zeros::<3>().len(), m::N);
    println!("{} {}", m::make::<u32>(), m::static_value(1.5f32));
    println!("{} {}", b::first(pair), m::S);
}

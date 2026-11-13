// `size_of` and `align_of` over a generic Adt application: the
// template check holds a placeholder and each instance answers with
// its own layout. Also a concrete application in a const item.
struct W<T> {
    a: u8,
    b: T,
}

struct V<T> {
    w: W<T>,
    c: u16,
}

enum E<T> {
    A(T),
    B(u8),
}

fn sz<T>() -> usize {
    std::mem::size_of::<W<T>>()
}

fn al<T>() -> usize {
    std::mem::align_of::<W<T>>()
}

fn nested<T>() -> usize {
    std::mem::size_of::<V<T>>()
}

fn esz<T>() -> usize {
    std::mem::size_of::<E<T>>()
}

const CW: usize = std::mem::size_of::<W<u32>>();

fn main() {
    println!("{} {} {} {}", sz::<u8>(), sz::<u32>(), sz::<u64>(), sz::<(u8, u16)>());
    println!("{} {}", al::<u8>(), al::<u64>());
    println!("{} {}", nested::<u8>(), nested::<u64>());
    println!("{} {}", esz::<u8>(), esz::<u64>());
    println!("{} {}", CW, std::mem::size_of::<W<u8>>());
}

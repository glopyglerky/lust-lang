// rel-front (2026-10-19): a stored discovery entry of `caller` asks for
// `make::<[u8; N]>`. A const fn edit moves N while no source of `caller`
// moves; v2 also edits another body, so the request takes a full resolve. The
// entry must replay its reads (const fn bodies join the generic environment).
const fn n_fn() -> usize {
    4
}

const N: usize = n_fn();

fn make<T: Default + Copy>() -> usize {
    let a = [T::default(); 2];
    a.len() + std::mem::size_of::<T>()
}

fn caller() -> usize {
    make::<[u8; N]>()
}

fn other() -> i64 {
    1
}

fn main() {
    println!("{} {}", caller(), other());
}

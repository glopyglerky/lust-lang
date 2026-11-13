// std fns named by a qualified path, with a turbofish, or by Rc/Arc paths,
// used as values: `<[T]>::len` (ts-rust ast/node.rs passes it for an
// `impl FnOnce(&[Node]) -> R`), `<str>::len`, `<Vec<T>>::new`, `<Box<T>>::new`,
// `Vec::<T>::with_capacity`, `mem::take::<T>`, `Arc::get_mut`
// (build_task.rs), and `Arc::unwrap_or_clone` (program_to_snapshot.rs).
use std::rc::Rc;
use std::sync::Arc;
fn read<R>(v: &[u32], f: impl FnOnce(&[u32]) -> R) -> R {
    f(v)
}
fn call0<R>(f: impl FnOnce() -> R) -> R {
    f()
}
fn call1<A, R>(f: impl FnOnce(A) -> R, a: A) -> R {
    f(a)
}
#[derive(Clone, Debug)]
struct Job(u32);
fn main() {
    let v = vec![1u32, 2, 3];
    let len: fn(&[u8]) -> usize = <[u8]>::len;
    println!("{} {} {} {}", read(&v, <[u32]>::len), read(&v[..0], <[u32]>::is_empty), <[u32]>::len(&v), len(b"ab"));
    let lens: Vec<usize> = vec![vec![1u8], vec![]].iter().map(Vec::as_slice).map(<[u8]>::len).collect();
    let empty: Vec<u16> = call0(<Vec<u16>>::new);
    let cap: Vec<u8> = call1(Vec::<u8>::with_capacity, 8);
    println!("{:?} {} {:?} {} {}", lens, call1(<str>::len, "abc"), empty, cap.capacity() >= 8, call1(<Box<u8>>::new, 4));
    let mut s = vec![5u8];
    let taken = call1(std::mem::take::<Vec<u8>>, &mut s);
    println!("{:?} {:?} {}", taken, s, call1(<Rc<u8>>::new, 6));
    let mut jobs = vec![Arc::new(Job(1)), Arc::new(Job(2))];
    let keep = jobs[1].clone();
    let got: Vec<Option<&mut Job>> = jobs.iter_mut().map(Arc::get_mut).collect();
    println!("{:?}", got);
    let mut one: Option<Arc<Job>> = Some(Arc::new(Job(3)));
    if let Some(j) = one.as_mut().and_then(Arc::get_mut) {
        j.0 += 1;
    }
    let owned: Vec<Job> = jobs.into_iter().chain(one).map(Arc::unwrap_or_clone).collect();
    println!("{:?} {}", owned, Arc::strong_count(&keep));
}

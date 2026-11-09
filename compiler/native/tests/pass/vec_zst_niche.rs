// A zero-sized element Vec stores capacity 0, as RawVec does, so the
// capacity word never holds an enclosing enum's niche value (fx-zstniche:
// `E::A(vec![N0], 1)` matched as `E::B`). capacity() still reports usize::MAX.
use std::collections::VecDeque;
use std::mem::size_of;

enum E<T> {
    A(Vec<T>, u8),
    B,
}
enum F {
    A(u8, Vec<()>),
    B,
    C,
    D,
}
struct N0;

fn show<T>(e: &E<T>) -> String {
    match e {
        E::A(v, n) => format!("A {} {}", v.len(), n),
        E::B => "B".to_string(),
    }
}

fn main() {
    let o: E<N0> = E::A(vec![N0], 1);
    println!("{}", show(&o));
    println!("{}", show(&E::<()>::A(Vec::new(), 2)));
    println!("{}", show(&E::<()>::B));
    let mut v: Vec<()> = Vec::with_capacity(3);
    v.push(());
    v.shrink_to_fit();
    println!("{}", show(&E::A(v, 3)));
    for f in [F::A(4, vec![(); 5]), F::B, F::C, F::D] {
        match f {
            F::A(n, v) => println!("F::A {} {}", n, v.len()),
            F::B => println!("F::B"),
            F::C => println!("F::C"),
            F::D => println!("F::D"),
        }
    }
    let d: Option<VecDeque<N0>> = Some(VecDeque::from(vec![N0, N0]));
    match d {
        Some(d) => println!("deque {}", d.len()),
        None => println!("no deque"),
    }
    let o: Option<Option<Vec<[u8; 0]>>> = Some(Some(vec![[]]));
    println!("{}", matches!(o, Some(Some(_))));
    println!("{} {} {}", size_of::<E<N0>>(), size_of::<F>(), size_of::<Option<Option<Vec<()>>>>());

    // Every capacity read reports usize::MAX for a zero-sized element.
    let mut v: Vec<()> = Vec::new();
    println!("{} {}", v.capacity(), v.spare_capacity_mut().len());
    v.resize(4, ());
    println!("{} {}", v.capacity(), v.spare_capacity_mut().len());
    let it = vec![(), (), ()].into_iter();
    let w: Vec<()> = it.collect();
    println!("{} {}", w.len(), w.capacity());
    let b: Box<[()]> = vec![(); 2].into_boxed_slice();
    let v = b.into_vec();
    println!("{} {} {}", v.len(), v.capacity(), matches!(Some(v), Some(_)));
    let u: Box<[std::mem::MaybeUninit<()>]> = Box::new_uninit_slice(6);
    println!("{}", u.len());
    let r: Vec<()> = (0..7).map(|_| ()).collect();
    println!("{} {}", r.len(), r.capacity());
}

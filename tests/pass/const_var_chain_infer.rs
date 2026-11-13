// A const argument inferred through two inference variables: the
// variant's `K<?a>` meets a struct literal field's `K<?b>`, and `?b` is
// fixed by the return type, which is the fn's own const parameter.
enum K<const N: usize> {
    H(u8),
    A([u8; N]),
}

struct S<const N: usize> {
    k: K<N>,
}

fn f<const N: usize>() -> S<N> {
    let k = K::H(1);
    S { k }
}

fn len<const N: usize>(s: &S<N>) -> usize {
    match &s.k {
        K::H(v) => *v as usize,
        K::A(a) => a.len() + N,
    }
}

fn main() {
    let s = f::<4>();
    let t = S::<2> { k: K::A([1, 2]) };
    println!("{} {}", len(&s), len(&t));
}

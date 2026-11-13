// A closure that uses a generic derive(Copy) Adt by value before collect
// made the Adt's instance: the capture is a copy (the closure is Fn), as
// the derive's bound `T: Copy` holds for its args. Was a tcc error: the
// closure fn took its env by value while its body read it by pointer.
#[derive(Clone, Copy, Debug)]
struct W<T>(T);
#[derive(Clone, Copy, Debug)]
struct V<A, B>(A, B);

fn call_fn<F: Fn()>(f: F) {
    f();
    f();
}

fn main() {
    let m = W(5u8);
    let c = move || drop(m);
    c();
    c();
    let v = V(W(1u8), W((2i64, 'c')));
    call_fn(move || println!("{:?}", v));
    let s = W(String::from("s"));
    let d = move || drop(s);
    d();
    println!("{} {:?}", m.0, v);
}

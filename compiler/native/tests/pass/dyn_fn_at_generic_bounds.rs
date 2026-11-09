// `dyn Fn` values meet generic Fn bounds through core's forwarding impls:
// `&dyn Fn`, `&Box<dyn Fn>`, `Box<dyn Fn>` by value, `&mut dyn FnMut` at
// `F: FnMut`, and `Box<dyn FnOnce>` at `F: FnOnce`.
fn apply<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 { f(x) }
fn apply_mut<F: FnMut(i32)>(mut f: F) { f(1); f(2); }
fn apply_once<F: FnOnce() -> String>(f: F) -> String { f() }
fn main() {
    let r: &dyn Fn(i32) -> i32 = &|x| x + 1;
    println!("{}", apply(r, 2));
    let b: Box<dyn Fn(i32) -> i32> = Box::new(|x| x * 3);
    println!("{} {}", apply(&b, 2), apply(&*b, 3));
    println!("{}", apply(b, 4));
    let mut log = Vec::new();
    {
        let m: &mut dyn FnMut(i32) = &mut |x| log.push(x);
        apply_mut(m);
    }
    println!("{:?}", log);
    let s = String::from("o");
    let o: Box<dyn FnOnce() -> String> = Box::new(move || s + "k");
    println!("{}", apply_once(o));
    let v: Vec<i32> = vec![1, 2].into_iter().map(|x| r(x)).collect();
    println!("{:?}", v);
}

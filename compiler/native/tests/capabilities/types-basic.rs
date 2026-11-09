// capability: types: bool
// capability: types: char
// capability: types: () unit
// capability: types: ! never
// capability: types: String
// capability: types: &str
// capability: types: Vec<T>
// capability: types: Option<T>
// capability: types: Result<T, E>
// capability: types: &T
// capability: types: &mut T
// capability: types: user structs and enums, with type, lifetime, and const generic parameters
// capability: types: f32 and f64
// capability: types: tuples
// capability: types: arrays [T; n]
// capability: types: slices &[T] and &mut [T]
// expect: ok

struct View<'a, T, const N: usize> {
    items: &'a [T; N],
}

enum Tree<T> {
    Leaf(T),
    Node(Box<Tree<T>>, Box<Tree<T>>),
}

fn sum(t: &Tree<u32>) -> u32 {
    match t {
        Tree::Leaf(v) => *v,
        Tree::Node(a, b) => sum(a) + sum(b),
    }
}

fn never() -> ! {
    panic!("unreachable")
}

fn bump(x: &mut u32) {
    *x += 1;
}

fn first(s: &mut [u32]) -> Option<&mut u32> {
    s.first_mut()
}

fn parse(s: &str) -> Result<u32, String> {
    s.parse::<u32>().map_err(|e| e.to_string())
}

fn main() {
    let b: bool = true;
    let c: char = 'x';
    let u: () = ();
    let s: String = String::from("hi");
    let r: &str = &s;
    let mut v: Vec<u32> = vec![1, 2];
    let o: Option<u32> = v.pop();
    let res: Result<u32, String> = parse("7");
    let mut n = 1u32;
    bump(&mut n);
    let arr: [u32; 3] = [4, 5, 6];
    let view = View { items: &arr };
    let sl: &[u32] = &arr[1..];
    let mut arr2 = [9u32, 8];
    if let Some(x) = first(&mut arr2[..]) {
        *x = 0;
    }
    let tup: (f32, f64) = (1.5, 2.25);
    let t = Tree::Node(Box::new(Tree::Leaf(1)), Box::new(Tree::Leaf(2)));
    let x: u32 = if b { 1 } else { never() };
    println!("{b} {c} {u:?} {r} {v:?} {o:?} {res:?} {n} {} {sl:?} {arr2:?} {tup:?} {} {x}", view.items.len(), sum(&t));
}

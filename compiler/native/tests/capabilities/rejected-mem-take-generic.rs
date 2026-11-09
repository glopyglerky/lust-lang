// capability: rejected: `mem::take` on this type (a generic `T: Default`)
// expect: L9001

fn f<T: Default>(x: &mut T) -> T {
    std::mem::take(x)
}

fn main() {
    println!("{}", f(&mut 3u32));
}

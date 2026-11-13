// Recursive types through Box: a linked list enum, layout
// sizes, and the Option<Box> niche.
#[derive(Debug)]
enum List { Cons(i32, Box<List>), Nil }
fn sum(l: &List) -> i32 {
    match l {
        List::Cons(n, next) => n + sum(next),
        List::Nil => 0,
    }
}
fn main() {
    let l = List::Cons(1, Box::new(List::Cons(2, Box::new(List::Cons(3, Box::new(List::Nil))))));
    println!("{} {:?}", sum(&l), l);
    let one: Option<Box<i32>> = Some(Box::new(7));
    println!("{:?} {}", one, std::mem::size_of::<Option<Box<i32>>>());
    println!("{}", std::mem::size_of::<Box<i32>>());
}
